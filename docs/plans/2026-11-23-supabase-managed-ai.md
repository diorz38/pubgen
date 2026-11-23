# Desktop side — Simentor service credentials for AI providers — mini plan

Companion to [`desktop-service-credentials-plan.md`](../../desktop-service-credentials-plan.md) (server side, source of truth). This plan covers only the Pubgen desktop app; schema, RLS, and Edge Functions live there.

## Goal

Add **Simentor Managed** as an independent option for AI translation (Groq) and cover generation (Cloudflare). The app signs in with Simentor Supabase Auth, enrolls its device with a locally generated keypair, fetches per-device encrypted credential ciphertext, decrypts at use time, and calls providers directly. Keep current Groq + Cloudflare, Local AI, and Custom AI workflows unchanged.

## Architecture

```text
Once per device:
  login (email/password, confirmed @bps.go.id)
    → generate libsodium keypair locally; private key stays in Windows secure storage
    → POST /functions/v1/device-enroll {action: register, device_label, public_key, key_alg, app_version}
    → super-admin grants service credential to user+device (Simentor web)
    → POST /functions/v1/service-credentials (fetch) with JWT → ciphertext + metadata

AI use (online or offline):
  cached ciphertext → crypto_box_seal_open with device private key (memory only)
    → direct provider request (Groq / Cloudflare)
```

- Server stores ciphertext + metadata only (`public.service_credentials`); no plaintext in the DB, no decrypt path server-side. Ciphertext is addressed to one enrolled device; a copied cache is useless on another device.
- App ships only Simentor Supabase URL + anon/publishable key. No `service_role`, no shared decrypt key, no hardcoded secrets.
- Enrollment auto-approves for confirmed `@bps.go.id` (identity by exact `auth_id`, never email-as-ownership). Credential access is a separate super-admin grant; enrollment alone grants nothing.
- Max 2 active devices per Simentor user. `409` from `device-enroll` shows: “Maksimal 2 perangkat aktif. Cabut perangkat lama di Simentor/Desktop Credential Manager.” Provide `list_mine` and `unregister` actions in the app.
- Translation and cover selections stay independent: managed translation uses the `groq` service credential, managed covers the `cloudflare` one. No automatic fallback between providers.

## Security boundary

- Fetch requires a valid authenticated session; **use does not**: cached ciphertext works offline, matching the server plan.
- Unregister, device/account revocation or deletion block future fetches only. An already-decrypted key stays usable until the upstream provider key is rotated — upstream rotation is the only kill switch; document this in the runbook and UI warnings.
- Private key lives in Windows secure storage (DPAPI/Credential Locker; TPM-backed CNG is the upgrade path, `ponytail:` note in code). Never uploaded, never in `Prefs`, `.env`, `.pubgen`, logs, or `Debug` output.
- Cache ciphertext + metadata only. Never write decrypted keys to disk; decrypt in memory at use time and clear after. No temp plaintext files.

## Crypto compatibility

- Sealed boxes: `crypto_box_seal` (issuance) / `crypto_box_seal_open` (desktop) — Curve25519 + XSalsa20-Poly1305.
- Rust implementation via a libsodium-compatible crate (e.g. `dryoc` pure-Rust or `libsodium-sys`); pick one, check its licence against the AGENTS.md allowlist before adding, and pin the exact version.
- Sealed boxes do not authenticate the sender; authorization and audit live in the issuance API, not in ciphertext.
- Cross-platform test vectors shared with Simentor before implementation; desktop decrypt must pass them before any fetch code lands.

## Implementation steps

1. **Auth:** email/password sign-in against the Simentor project, sign-out, session restore/refresh; session tokens in Windows Credential Manager; network off the UI thread; wasm stubs return unavailable.
2. **Device keypair + enrollment:** generate the keypair on first run; store the private key in the credential vault; call `device-enroll` register/list_mine/unregister after login; handle the 2-device `409`.
3. **Credential fetch + cache:** call `service-credentials` with JWT; store returned ciphertext + metadata in a local cache (ciphertext is safe at rest); refresh on login or manual sync.
4. **Decrypt-at-use plumbing:** load the private key from the vault, `crypto_box_seal_open` in memory, use, clear; redacted errors; never a plaintext-at-rest path.
5. **Provider options:** add the managed choice to the translation selector and cover setup; resolve the managed credential through steps 3–4 and reuse the existing Groq transport and Cloudflare cover path. Leave Local, Custom, and user-configured Groq/Cloudflare untouched.
6. **Ops:** rotation/compromise runbook (revoke blocks fetches; rotate upstream key to kill issued ones), no-secret audit/diagnostics, portable x64 Windows packaging only.

## Verification

- Unit tests: sealed-box decrypt against published test vectors; 409/unregister/list_mine handling; ciphertext-only cache (assert no plaintext key bytes on disk after a use); `Debug`/error redaction; provider selection independence.
- Regression: existing Groq, Custom, Local AI, direct Cloudflare, and offline workflows unchanged without Simentor.
- End-to-end staging (server plan Phase 4): BPS login → auto-enroll → super-admin grant → fetch ciphertext → decrypt → use; copied ciphertext fails on another device; revoke/unregister blocks new fetches but not local use; upstream rotation invalidates old keys.

## Not in scope

- Server schema, RLS, Edge Functions, and the super-admin web page (see the server plan).
- Guaranteeing keys are unusable outside this app after download; upstream rotation is the enforcement point.
- Separate app-level per-user quotas (upstream provider limits apply, shared across eligible users).
- Linux/macOS vaults (portable x64 Windows only for this release).

## Open items (server side will propose)

- Simentor project URL + publishable key distribution for the shipped binary.
- Exact `service_name` values for the two managed credentials (assumed `groq`, `cloudflare`).
- Shared sealed-box test vectors and pinned libsodium version compatibility.
- UX copy and timing for “waiting for super-admin grant” state before first fetch is possible.
