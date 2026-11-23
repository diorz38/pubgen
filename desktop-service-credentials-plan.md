# Desktop Service Credentials — Mini Plan

## Goal

Store service credentials in Supabase as **ciphertext**; Simentor web manages issuance; desktop app decrypts locally with a device private key. One-time login/download is enough for offline use because ciphertext is cached locally and decrypted only on the enrolled device.

## Locked Decisions

- **Encryption model:** per-device public-key encryption. Admin issuance encrypts to device **public key**; desktop decrypts with device **private key**. No shared hardcoded decrypt key.
- **Auto-approve:** valid Supabase Auth login + `@bps.go.id` email auto-approves **device enrollment** only. It does not auto-grant service credentials. Dedicated service-key management page and privileged management actions are super-admin-only.
- **Device limit:** max **2 active (non-revoked) devices per Simentor user**. Unregistering frees a slot. Enforced atomically in the database and checked by Edge Function.
- **No plaintext at rest in Supabase:** only ciphertext + metadata. No `VITE_*` secrets, no plaintext key columns, no DB-side decrypt path for normal clients.
- **Credential entitlement:** super-admin grants a credential to a user/device. Enrollment alone grants no credential access. “Access obtained” means credential ciphertext downloaded and decrypted on device; after that, credential remains usable even after unregister, account deletion, or server-side revocation. Those actions block future downloads only. Upstream service-key rotation is the only kill switch for already-issued credentials.
- **Desktop auth:** planned desktop app signs in with Simentor Supabase Auth and uses its JWT to enroll and fetch ciphertext. Desktop implementation is part of this plan; it is not present in the repo yet.

## Phase 1 — Schema, RLS, DB Guards

**Scope**
- Migration: `supabase/migrations/20261123xxxxxx_desktop_service_credentials.sql`.
- `public.app_devices`: `id uuid pk default gen_random_uuid()`, `user_id bigint not null references public.users(id) on delete cascade`, `auth_user_id uuid`, `device_label text`, `device_fingerprint text`, `public_key text not null`, `key_alg text not null`, `app_version text`, `approval_mode text not null default 'auto_bps'`, `approved_at timestamptz`, `revoked_at timestamptz`, `last_seen_at timestamptz`, `created_at timestamptz default now()`, `updated_at timestamptz default now()`.
- `public.service_credentials`: `id uuid pk default gen_random_uuid()`, `owner_user_id bigint not null references public.users(id) on delete cascade`, `recipient_device_id uuid not null references public.app_devices(id) on delete cascade`, `service_name text not null`, `credential_name text not null`, `ciphertext text not null`, `cipher_alg text not null`, `key_version int not null default 1`, `created_by bigint references public.users(id)`, `created_at timestamptz default now()`, `revoked_at timestamptz`. Physical account/device deletion may remove server ciphertext/metadata; it never removes local key material or disables an already-acquired service key.
- Optional `public.service_credential_audit`: actor, action, target ids, metadata jsonb, timestamp; no secret/plaintext fields.
- DB guard: enforce max two active rows per `user_id` **without a count-then-insert race**. Use a per-user transaction advisory lock in trigger/function (and acquire it on relevant user/revocation changes), or a two-slot model with unique constraints. Edge Function check improves error clarity; DB is final guard.
- RLS/grants: follow repo pattern (`20260812_restrict_all_quiz_tables_by_email_domain.sql` style): `ENABLE ROW LEVEL SECURITY`; idempotent name-agnostic policy cleanup; explicitly `REVOKE ALL ON TABLE ... FROM anon, authenticated`, then grant only required operations. Authenticated clients need own-row `SELECT` only if UI requires direct reads; otherwise access through user-authenticated Edge Functions. Test anon and authenticated allow/deny behavior.
- Policies: `TO authenticated`, require exact Simentor ownership (`public.users.auth_id = (select auth.uid())`); email match is not an ownership fallback. `app_devices` exposes own metadata only. `service_credentials` exposes ciphertext only for rows owned by caller and addressed to their active device; never grant access based only on email domain. No authenticated writes for privileged issuance paths.
- Credential reads: Edge Function validates JWT, resolves caller's `public.users` row by `auth_id`, verifies recipient device belongs to caller and is active, and returns only ciphertext+metadata for that device. Credential grant is explicit and separate from device enrollment.

**Acceptance**
- [ ] Migration applies cleanly and is idempotent.
- [ ] `anon` has no access to both tables.
- [ ] Authenticated `@bps.go.id` user reads only own device/credential rows; non-BPS and unrelated authenticated users read none.
- [ ] Third active device for same Simentor user is rejected by DB; unregistering/revoking one allows a new device.
- [ ] Schema contains no plaintext service-key column; audit cannot store plaintext.

## Phase 2 — Device Enrollment Edge Function

**Scope**
- New Edge Function: `supabase/functions/device-enroll/index.ts`, patterned after `admin-users/index.ts`.
- Verify caller with `createClient(url, anonKey, { global: { headers: { Authorization } } }).auth.getUser()`.
- Resolve `public.users` profile by exact `auth_id` only; require caller's verified Auth email to end in `@bps.go.id` (case-insensitive). Do not use email as identity fallback.
- `register`: validate `device_label`, `public_key`, `key_alg`, `app_version`, optional fingerprint. Reject private/secret fields. Insert active auto-approved device.
- Enforce max 2 active devices before insert; DB atomic guard is final. Return `409` with clear message when limit reached.
- `list_mine`: return own devices/slot state.
- `unregister`: authenticated user explicitly unregisters own device by `device_id`; set `revoked_at` (soft revoke), retain audit history, free one active slot, and block future fetches. Idempotent response for an already-unregistered own device; reject unknown/other-user IDs without leaking ownership.
- `admin_revoke`: super-admin can revoke any device; retain audit history and block future fetches.
- Revoking/unregistering does not erase already-downloaded ciphertext or locally cached private key and cannot disable already-issued service keys; document this limit in UI/runbook.
- Deploy: `supabase functions deploy device-enroll --project-ref <ref> --use-api`.

**Acceptance**
- [ ] Valid, verified `@bps.go.id` login registers device without admin approval; identity resolves by exact `auth_id`.
- [ ] Invalid JWT/non-BPS/over-limit requests rejected with clear errors.
- [ ] Max-2 active-device limit per Simentor user enforced server-side and atomically DB-side, including concurrent registration attempts.
- [ ] `unregister` endpoint revokes only caller's device, is idempotent for own already-unregistered device, rejects cross-user IDs, and frees a slot.
- [ ] Admin revoke blocks future fetches; neither unregister nor revoke disables already-downloaded keys.
- [ ] No private key/plaintext credential appears in logs or responses.

## Phase 3 — Dedicated Super-Admin Service Keys Page

**Scope**
- Add one dedicated page/module solely for desktop service-key management: `src/views/AdminDesktopCredentials.tsx` (or equivalent), route in `src/router.tsx`, navigation module in `src/constants.tsx`, service in `src/lib/supabase-desktop-credentials.ts`, export via `src/lib/api-services.ts`.
- Page access restricted to users with exact `super-admin` role. Enforce in route/navigation UI and independently in every privileged Edge Function; hiding the link is not authorization. Non-super-admin users must be denied even with direct URL/API calls.
- Page includes device management (user/device, status, approval, last seen, app version, revoke/unregister status, active slots) and service-key management (issue/revoke, select owner and their active enrolled device, inspect metadata/version/status).
- Super-admin explicitly grants each service key to a user/device. Verify selected device belongs to selected owner and is active. Enrollment alone never grants keys.
- Encrypt to enrolled active device's public key; preferred browser-side encryption, upload ciphertext only. Trusted service-role Edge Function encryption is fallback only; plaintext must never be persisted or logged.
- Privileged writes via `supabase/functions/service-credentials/index.ts` with server-side super-admin check like `admin-users`.
- Page never displays saved plaintext; show metadata/version/status only.

**Acceptance**
- [ ] Dedicated page manages only desktop device/service-key feature and is visible/openable only to `super-admin`.
- [ ] Non-super-admin denied by route guard and server checks, including direct URL and direct Edge Function calls.
- [ ] Super-admin explicitly grants credentials to a user/device; enrollment never grants credentials implicitly.
- [ ] Stored credential rows contain ciphertext + metadata only.
- [ ] Ciphertext bound to specific owner/device; cross-user/device reads denied by RLS/API.
- [ ] Revoke/unregister updates server status; subsequent desktop fetch excludes revoked rows, while already-downloaded credentials remain usable until upstream key rotation.

## Phase 4 — Desktop Integration, Rotation, Hardening

**Scope**
- Planned desktop app uses Simentor Supabase Auth; implements enrollment, explicit credential fetch, local ciphertext cache, local decrypt-on-use.
- Add refresh behavior on login/manual sync; offline uses cached ciphertext. Once downloaded, keys remain usable offline even after account/device/credential revocation or deletion.
- Define compromise response: revoke device/credential in Simentor to block future fetches, rotate upstream service key to invalidate keys already issued, issue new ciphertext version.
- Security hardening: signed installer/build, no plaintext disk writes, memory hygiene, audit events without secrets.

**Acceptance**
- [ ] End-to-end: BPS login -> auto-enroll -> fetch ciphertext -> decrypt locally -> use credential.
- [ ] Copied ciphertext useless on another device.
- [ ] 2-device limit and revoke/re-enroll flow works.
- [ ] Device unregister/revoke and credential revoke block subsequent fetches; already-downloaded credentials remain usable until upstream key rotation.
- [ ] Upstream rotation runbook documented and tested; account deletion does not implicitly revoke or disable locally held keys.

## Desktop App Dev — Implement Now

**Required design**

1. **Per-device keypair on device.** Generate locally on first run. Public key may be uploaded; private key never leaves device.
2. **Private key in Windows secure storage.** DPAPI/Credential Locker; prefer TPM-backed CNG key where possible. No config/plaintext file/log.
3. **Enrollment API call after Supabase login.** Send `device_label`, `public_key`, `key_alg`, `app_version`, optional fingerprint. Expect auto-approve for `@bps.go.id`.
4. **Unregister endpoint.** Provide authenticated `unregister` action so user can remove current/selected device from active slots; clearly warn this blocks future fetches but does not disable keys already downloaded.
5. **Handle 2-device limit.** If server returns max-device/409, show: “Maksimal 2 perangkat aktif. Cabut perangkat lama di Simentor/Desktop Credential Manager.”
6. **Cache ciphertext only.** Local cache can store ciphertext+metadata for offline use. Never cache decrypted service keys on disk.
7. **Decrypt at use time only.** Load private key from secure storage, decrypt in memory, use, clear. No temp plaintext files.
8. **Crypto compatibility.** Use libsodium sealed boxes (`crypto_box_seal` / `crypto_box_seal_open`): Curve25519 key agreement with XSalsa20-Poly1305 authenticated encryption. Same format supported by browser issuance and desktop decryption; admin can encrypt to public key while device is offline. Pin exact libsodium versions and publish cross-platform test vectors before implementation. Sealed boxes do not authenticate the sender; super-admin authorization and audit happen in the issuance API, not in ciphertext.
9. **Server-trusted identity.** Use Supabase JWT on every request. Do not rely on client-side “trusted app” flags.

**Suggested device contract**

```json
POST /functions/v1/device-enroll
{
  "action": "register",
  "device_label": "PC-KEU-01",
  "public_key": "base64...",
  "key_alg": "libsodium-sealed-box",
  "app_version": "1.0.0",
  "device_fingerprint": "optional-nonsecret-id"
}
```

```json
{
  "device_id": "uuid",
  "status": "active",
  "approval_mode": "auto_bps",
  "active_device_count": 1,
  "max_active_devices": 2
}
```

**Do not do**

- Do not hardcode a shared decrypt key in the desktop app.
- Do not upload private keys.
- Do not save decrypted service keys to disk.
- Do not assume offline revocation works; upstream key rotation is the real kill switch.

## Repo Implementation Notes

- Follow existing Supabase Auth + RLS conventions: `src/lib/supabase.ts`, `src/lib/auth.ts`, `src/lib/authz.ts`.
- Privileged writes follow `supabase/functions/admin-users/index.ts`: JWT caller verification, role check, service-role DB client, clear JSON errors.
- RLS migrations follow name-agnostic/idempotent patterns in recent `supabase/migrations/20260812_*.sql`.
- Before schema changes: read Supabase skill/checklist, consider RLS/privileges, and run advisors/security checks when applying.