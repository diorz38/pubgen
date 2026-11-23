# Supabase-distributed AI keys — mini plan

## Goal

Add **Supabase Managed** as an independent option for AI translation and cover generation. On first setup, a signed-in user downloads the assigned provider credentials once. The app stores credentials in the operating-system secure credential store and uses them for direct provider requests. Keep current Groq + Cloudflare, Local AI, and Custom AI workflows unchanged.

## Security boundary — read before implementation

This design distributes usable provider credentials to desktop users. It cannot guarantee that credentials are usable only inside this app: once the app can decrypt/use a key, an authorized user or malware on that user's PC can extract it and call the provider outside the app. OS secure storage protects credentials at rest, not from their owner or a compromised account.

A login/session check can gate downloading keys, but cannot revoke a downloaded key or make it stop working after logout/session expiry. A one-time fetch therefore means **offline reuse until the provider key is rotated/revoked**, not continued Supabase-enforced authorization. If strict app-only use, immediate logout enforcement, or central per-user limits are required, use the server-side AI proxy architecture instead; do not distribute provider keys.

## Architecture

```text
First setup:
Pubgen Studio ── Supabase Auth + HTTPS ──> Edge Function ──> authorized provider credentials
       └── store credentials in OS credential vault

AI use:
Pubgen Studio ── provider request + locally retrieved credential ──> Groq / Cloudflare
```

- Use an authenticated Edge Function as the only key-delivery endpoint; app does not connect to Postgres or read key tables directly. Store shared provider credentials as Edge Function secrets, not in an exposed database table. Function validates the user's JWT and confirmed email, then requires the normalized email domain to equal exactly `bps.go.id`. Email eligibility alone grants access; no separate entitlement is planned. Never authorize from editable `user_metadata`.
- Keep Supabase project URL + publishable key in the app; never ship `service_role`, secret keys, or provider keys in the binary.
- Deliver keys only over HTTPS. Store downloaded keys in Windows Credential Manager/DPAPI. Do not add custom client-side encryption with a decryption key embedded in the app; OS storage protects keys at rest but not from the authorized PC user.
- Store each credential separately by provider; never save keys in `Prefs`, `.env`, `.pubgen`, ordinary settings files, logs, or error text. Redact credentials in `Debug` output.
- Use separate managed selections for translation and covers. Translation uses the managed Groq credential; cover generation uses the managed Cloudflare credential. No automatic fallback between providers.
- Assume credentials are shared across eligible app users unless providers support issuing per-user keys. Shared credentials mean one leak can affect all users; define rotation and abuse response before release.

## Authentication and local behavior

- Sign-in is required to retrieve/download managed keys. Supabase Auth refresh-token/session storage must use the OS credential vault.
- Fetch keys only during authenticated setup/refresh—not for each AI request. Store them in Windows Credential Manager, which encrypts secrets using OS-protected storage. The app reads a managed key into memory only while a valid signed-in session exists; on sign-out/session expiry, clear plaintext keys from memory and block managed AI use. Keep encrypted vault entries for later signed-in use; require fresh login before reading them again. This app-level gate cannot stop a user extracting keys.
- Do not promise 12-hour session expiry on Free tier. Default Auth sessions may persist/refresh. Session validity gates managed use inside the app, but cannot revoke keys copied from the PC; provider-side revocation is needed to invalidate copied keys.
- Target is the portable x64 Windows app only. Use Windows Credential Manager/DPAPI-backed secure storage; no Linux/macOS vault support is needed for this release. Fail closed if Windows secure storage is unavailable; do not fall back to plaintext.

## Implementation steps

1. **Auth and key retrieval:** Add email/password sign-in, sign-out, and session restore. Require confirmed email and normalized domain exactly `bps.go.id`. Implement an Edge Function that validates JWT and domain eligibility, retrieves configured shared provider credentials server-side, and returns only the required credentials. Keep provider secrets server-side and key tables unavailable to direct Data API roles.
2. **Secure local storage:** Add a Windows Credential Manager/DPAPI-backed vault for provider keys and Supabase refresh credentials. Add save/read/delete behavior and clear UI for missing, locked, revoked, or stale credentials. Never serialize secrets into application prefs or project files.
3. **Translation option:** Add Supabase Managed choice to the translation selector. Fetch keys on explicit setup/refresh; resolve the managed key from the vault and reuse existing online Groq transport. Leave Local, Custom, and user-configured Groq unchanged.
4. **Cover option:** Add Supabase Managed choice to cover setup. Resolve the managed Cloudflare credential from the vault and reuse the current cover generation path. Leave direct user-configured Cloudflare unchanged.
5. **Operations and release:** Treat upstream provider quotas/rate limits as usage limits; add no separate per-user quota initially. Add provider-key rotation/revocation runbook, abuse response, TLS requirements, and redacted diagnostics. Explain that downloaded credentials are OS-encrypted at rest but extractable by the authorized PC user; the app blocks their use after logout/session expiry, but cannot disable copies outside the app. Package/test for portable x64 Windows only.

## Verification

- Unit tests: provider selection remains independent; signed-out/expired session cannot read vault keys or start managed requests; sign-out clears plaintext keys from memory but retains encrypted vault entries; vault round-trip/delete; no secret in serialized prefs, package files, logs, `Debug`, or errors.
- Edge Function tests: reject missing/invalid JWT, unconfirmed email, and non-`@bps.go.id` user; return only configured credentials; reject unknown provider names; never log secrets. Verify app has no direct key-table access.
- Regression: existing Groq, Custom, Local AI, direct Cloudflare, and offline workflows continue unchanged without Supabase.
- Manual staging test: sign in → retrieve/store keys → restart app → sign in and generate translation/cover without another Supabase key fetch → sign out/session expire and confirm managed use is blocked → rotate/revoke provider key and confirm copied old keys stop working only after provider revocation.

## Not in scope

- Guaranteeing provider keys are usable only inside this app or inaccessible to the authorized PC user.
- Immediate server-side revocation of a previously downloaded key through Supabase logout.
- Guaranteed 12-hour session limit on unsupported Supabase plans.
- Billing, automatic cross-provider failover, or plaintext key storage as a fallback.

## Confirmed choices

- Sign-in: email/password with confirmed email.
- Eligibility: confirmed email domain exactly `bps.go.id`; no separate entitlement.
- Usage limits: upstream provider quotas/rate limits; no additional app per-user cap initially.
- Distribution: portable x64 Windows only; Windows Credential Manager/DPAPI.
- Local cache: keep encrypted vault entries; require active sign-in before the app reads/uses keys; clear plaintext keys from memory and block managed AI after sign-out/session expiry.

## Remaining operational decision

- Provider keys appear to be shared app-level credentials. Confirm upstream key type and document shared quota, rotation owner, and abuse response before release. Even if the app gates use on sign-in, any user can extract and reuse a downloaded key outside the app; use Edge Function proxying if that risk is unacceptable.
