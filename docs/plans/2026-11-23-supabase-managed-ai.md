# Desktop side — Simentor service credentials for AI providers — mini plan

Companion to [`desktop-service-credentials-plan.md`](../../desktop-service-credentials-plan.md) (server side) and its integration contract [`DESKTOP_TEAM_HANDOFF.md`](../../DESKTOP_TEAM_HANDOFF.md), which wins on every wire detail. This plan covers only the Pubgen desktop app; schema, RLS, and Edge Functions live on the server side.

## Goal

Add **Simentor Managed** as an independent option for AI translation (Groq) and cover generation (Cloudflare). The app signs in with Simentor Supabase Auth, enrolls its device with a locally generated keypair, fetches per-device encrypted credential ciphertext, decrypts at use time, and calls providers directly. Keep current Groq + Cloudflare, Local AI, and Custom AI workflows unchanged.

## Architecture

```text
Once per device:
  login (email/password, verified @bps.go.id, linked public.users row)
    → generate libsodium keypair locally; private key stays in the Windows vault
    → POST /functions/v1/device-enroll {action: register, device_label, public_key, key_alg, app_version}
    → super-admin issues a credential to this user+device (Simentor web)
    → POST /functions/v1/service-credentials {action: fetch_mine, device_id} with JWT → ciphertext + metadata

AI use (online or offline):
  cached ciphertext → crypto_box_seal_open with device private key (memory only)
    → direct provider request (Groq / Cloudflare)
```

- Configuration is `appsettings.json` beside the executable (`SupabaseUrl`, `PublishableKey`), per handoff §2; a source checkout may instead set `SIMENTOR_URL` / `SIMENTOR_ANON_KEY`. HTTPS only (loopback HTTP for a local stack). Never a `service_role` key.
- Server stores ciphertext + metadata only; no plaintext in the DB, no decrypt path server-side. Ciphertext is sealed to one enrolled device; a copied cache is useless on another PC.
- Enrollment auto-approves for a verified `@bps.go.id` account (identity by exact `auth_id`, never email-as-ownership). Enrollment grants **no** credential: a super-admin issues each one.
- Max 2 active devices per Simentor user; `409` is authoritative (`active_device_count` may be stale). The app shows the handoff's exact `409` message and offers `unregister`.
- Translation and cover selections stay independent: managed translation uses the `groq` service credential, managed covers the `cloudflare` one. No automatic fallback between providers.

## Security boundary

- Fetch requires a valid authenticated session; **use does not**: cached ciphertext works offline, matching the server plan.
- Unregister, device/credential/account revocation and deletion block future fetches only. An already-downloaded key stays usable until the upstream provider key is rotated — upstream rotation is the only kill switch. The UI states this permanently, not only in a modal.
- Private key and refresh token live in Windows Credential Manager (DPAPI, current user). Never uploaded, never in `Prefs`, `.env`, `.pubgen`, logs, or `Debug` output.
- The ciphertext cache is encrypted at rest with a ChaCha20-Poly1305 key held in the same vault (handoff §6), and holds ciphertext + metadata only. Plaintext exists in memory just long enough to use it; no temp files, no clipboard write without an explicit user action.
- Losing the Windows profile or the vault entries means the cache is unrecoverable: the fix is unregister + re-enroll + re-issue, not key recovery.

## Crypto compatibility

- Sealed boxes: `crypto_box_seal` (issuance) / `crypto_box_seal_open` (desktop) — Curve25519 + XSalsa20-Poly1305.
- Rust implementation via a libsodium-compatible crate (e.g. `dryoc` pure-Rust or `libsodium-sys`); pick one, check its licence against the AGENTS.md allowlist before adding, and pin the exact version.
- Sealed boxes do not authenticate the sender; authorization and audit live in the issuance API, not in ciphertext.
- Cross-platform test vectors shared with Simentor before implementation; desktop decrypt must pass them before any fetch code lands.

## Implementation steps

1. **Auth:** email/password sign-in against the Simentor project, sign-out, session restore/refresh; session tokens in Windows Credential Manager; network off the UI thread; wasm stubs return unavailable.
2. **Device keypair + enrollment:** generate the keypair on first run; store the private key in the credential vault; call `device-enroll` register/list_mine/unregister after login; handle the 2-device `409`.
3. **Credential fetch + cache:** `service-credentials {action: fetch_mine, device_id}` with the JWT; cache ciphertext + metadata encrypted at rest with a vault-held AEAD key; refresh on login or explicit sync.
4. **Decrypt-at-use plumbing:** load the private key from the vault, `crypto_box_seal_open` in memory, use, clear; redacted errors; never a plaintext-at-rest path.
5. **Provider options:** add the managed choice to the translation selector and cover setup; resolve the managed credential through steps 3–4 and reuse the existing Groq transport and Cloudflare cover path. Leave Local, Custom, and user-configured Groq/Cloudflare untouched.
6. **UX (handoff §8):** email/password login with the BPS hint; auto-enrol after login with slot usage (`1/2`); the exact `409` message; cached credential list marked `Offline · locked` until a decrypt succeeds, with `Copy` enabled only then; `Unregister device` with the revocation warning; and the revocation caveat shown permanently.
7. **Ops:** rotation/compromise runbook (revoke blocks fetches; rotate upstream key to kill issued ones), no-secret audit/diagnostics, portable x64 Windows packaging only.

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

- **Deployment status (2026-11-23):** project `https://gwkowacuvtnaapzkqcnf.supabase.co` answers `/auth/v1/settings` with the project's anon key and email auth enabled, but `device-enroll` and `service-credentials` both return `404 Requested function was not found`. The migration and both functions still have to be deployed before any end-to-end run. Verified with `crates/simentor/tests/live.rs` (`-- --ignored`).

- Delivery of `appsettings.json` (`SupabaseUrl`, `PublishableKey`) with the shipped portable build, and who updates it when the project changes (handoff §12.1).
- Exact `service_name` values for the two managed credentials (assumed `groq`, `cloudflare`); the issuer must encrypt the Cloudflare one as `{"account_id","api_token"}` JSON.
- Shared sealed-box test vectors and pinned libsodium version compatibility.
- Whether the app should adopt a `last_seen_at` heartbeat action (handoff §12.5) — not implemented; the server currently sets it at registration only.
