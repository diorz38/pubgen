# Desktop Team Handoff — Service Credentials

Everything the desktop team needs to build or replace the Windows client. Source of truth for behavior is the code in this branch; this document is the integration contract.

Related: [plan](../plans/desktop-service-credentials-plan.md), [rotation runbook](DESKTOP_SERVICE_CREDENTIALS_RUNBOOK.md), reference client in [`desktop/Simentor.Desktop/`](../desktop/Simentor.Desktop/README.md).

---

## 1. What this feature is (and is not)

**Is:** a one-time, per-device delivery channel for service credentials. Simentor web issues a credential sealed to a specific enrolled device; the desktop fetches ciphertext, caches it, and decrypts only in memory when it needs the value.

**Is not:** a remote kill switch. Once a credential is downloaded and decrypted, revoking the device, revoking the credential, or deleting the Simentor account **does not** invalidate it. Only rotating the key at the upstream service does. Tell users this before they enrol.

---

## 2. Environment the team must be given

| Item | Value |
| --- | --- |
| Supabase project URL | `https://<ref>.supabase.co` (HTTPS only) |
| Public key | `sb_publishable_...` or the project anon key |
| Secret / service_role key | **Never** — desktop must not hold it |
| Functions | `device-enroll`, `service-credentials` deployed |
| Migration | `20261123110000_desktop_service_credentials.sql` applied |

Desktop config file `appsettings.json` beside the executable:

```json
{ "SupabaseUrl": "https://<ref>.supabase.co", "PublishableKey": "sb_publishable_..." }
```

Ship `appsettings.example.json` and gitignore the real one (already done in this repo).

Reference stack used here: .NET 8 (`net8.0-windows`), WPF, `Sodium.Core` 1.4.1 (brings native `libsodium` 1.0.22), `System.Security.Cryptography.ProtectedData` 8.0.0. Any stack is fine if sections 5–7 hold.

---

## 3. Authentication

Supabase Auth, email + password, one interactive login per Windows user.

```
POST {SUPABASE_URL}/auth/v1/token?grant_type=password
apikey: <publishable key>
{ "email": "...", "password": "..." }
```

Refresh (used on app start to restore the session):

```
POST {SUPABASE_URL}/auth/v1/token?grant_type=refresh_token
apikey: <publishable key>
{ "refresh_token": "<token>" }
```

Rules enforced server-side; the client should pre-check to fail fast:

- Email must be **verified** (`email_confirmed_at` non-null) and end in `@bps.go.id` (case-insensitive).
- The Auth user must be linked to a `public.users` row by exact `auth_id`. A login with no Simentor profile gets `403`.
- Note: the refresh response may omit the `user` object on some Auth versions. Keep the stored user identity and re-check only the email suffix.

Every Edge Function call needs **both** headers:

```
apikey: <publishable key>
Authorization: Bearer <access_token>
```

---

## 4. API contract

All endpoints are `POST` to `{SUPABASE_URL}/functions/v1/<fn>` with a JSON body containing `action`. Responses are JSON with `success: boolean`; failures also carry `message`.

### 4.1 `device-enroll`

#### `register` — enrol this device

```json
{
  "action": "register",
  "device_label": "PC-KEU-01",
  "public_key": "<standard base64, exactly 32 bytes decoded>",
  "key_alg": "libsodium-sealed-box",
  "app_version": "1.0.0",
  "device_fingerprint": "optional, non-secret, <=200 chars"
}
```

Success `200`:

```json
{
  "success": true,
  "device_id": "<uuid>",
  "status": "active",
  "approval_mode": "auto_bps",
  "active_device_count": 1,
  "max_active_devices": 2,
  "device": { "id": "<uuid>", "...": "full row" }
}
```

Validation: `device_label` 1–120 chars, `app_version` 1–40 chars, `key_alg` must equal `libsodium-sealed-box`, `public_key` must decode to exactly 32 bytes. Request is rejected if it carries `private_key`, `secret_key`, `credential`, or `plaintext`.

`409` means the 2-active-device cap. Show: *"Maksimal 2 perangkat aktif. Cabut perangkat lama di Simentor/Desktop Credential Manager."*

> `active_device_count` is computed before insert and can be stale under a concurrent registration; treat it as informational and trust `409` instead.

#### `list_mine` — own devices

```json
{ "action": "list_mine" }
```

Returns `devices: [...]` where each row has **`id`** (not `device_id`) plus `device_label`, `public_key`, `key_alg`, `app_version`, `approval_mode`, `approved_at`, `last_seen_at`, `created_at`, `revoked_at`, and `active_device_count` / `max_active_devices` at the top level.

Gotcha: `register` returns `device_id`, `list_mine` returns `id`. Normalize on the client.

#### `unregister` — remove this device (frees a slot)

```json
{ "action": "unregister", "device_id": "<uuid>" }
```

`200` with `{ success, device_id, status: "revoked", already_unregistered: bool }`. Idempotent for your own device. Another user's device id returns `404` (no ownership leak).

#### Admin-only actions

`list_all` and `admin_revoke` require the `super-admin` role and `403` otherwise. `list_all` adds `user_name` / `user_email` and a `status` field. Desktop never calls these.

### 4.2 `service-credentials`

#### `fetch_mine` — get ciphertext for this device

```json
{ "action": "fetch_mine", "device_id": "<uuid>" }
```

```json
{
  "success": true,
  "credentials": [
    {
      "id": "<uuid>",
      "owner_user_id": 7,
      "recipient_device_id": "<uuid>",
      "service_name": "API Internal",
      "credential_name": "Production key",
      "ciphertext": "<standard base64>",
      "cipher_alg": "libsodium-sealed-box",
      "key_version": 1,
      "created_at": "2026-11-23T00:00:00+00:00"
    }
  ]
}
```

Only unrevoked credentials addressed to this device, newest first. Requires a verified BPS login, an active device that belongs to the caller, and matching `auth_user_id`. Revoked device → `403`. Each call writes a `credential.fetch` audit row (no secret content).

#### Admin-only: `list_all`, `issue`, `revoke`

Metadata-only listing; `issue` takes `owner_user_id`, `recipient_device_id`, `service_name`, `credential_name`, `plaintext` (1–65536 chars) and returns metadata only; `revoke` takes `credential_id`. All super-admin. Desktop never calls these — issuance happens in the Simentor web page.

### 4.3 Status codes

| Code | Meaning on the client |
| --- | --- |
| 400 | Bad payload (label/app version/key length/bad base64) — fix the request, do not retry |
| 401 | Missing/expired JWT — refresh, else re-login |
| 403 | Not verified BPS, no Simentor profile, device revoked, or not super-admin |
| 404 | Device not found or not yours |
| 409 | Device cap reached (register only) |
| 500 | Server/DB error — safe to retry once, then surface a message |

Never retry blindly after `409` or `400`.

---

## 5. Cryptography contract (must match exactly)

- Algorithm: **libsodium sealed box** — `crypto_box_seal` / `crypto_box_seal_open` (Curve25519 + XSalsa20-Poly1305).
- `key_alg` / `cipher_alg` string: `libsodium-sealed-box`.
- Keypair: 32-byte public, 32-byte secret. Generate on device; treat as immutable for the device lifetime.
- Encoding: **standard base64 with padding** (`sodium.base64_variants.ORIGINAL`, C# `Convert.ToBase64String`). No URL-safe alphabet, no hex.
- Ciphertext length = plaintext bytes + 48 (32 ephemeral public key + 16 MAC). The client does not need to parse it; just base64-decode the whole string and open it.
- The server re-encodes the public key canonically, so string comparisons on `public_key` are safe only for standard base64.
- Sealed boxes are **anonymous**: they prove nothing about who encrypted. Authenticity comes from the super-admin check and the audit trail server-side.

Self-test to include in every build (this repo runs it at startup): generate a keypair, seal a known plaintext, open it, compare constant-time.

---

## 6. Key and cache storage rules

| Data | Rule |
| --- | --- |
| Device private key | Windows DPAPI, `DataProtectionScope.CurrentUser`. No plaintext file, no registry, no log. |
| Auth session (access + refresh token) | DPAPI `CurrentUser` as well. |
| Ciphertext cache | DPAPI `CurrentUser`. Ciphertext + metadata only. |
| Decrypted credential | Memory only, zeroed after use. Never written to disk, temp files, crash dumps, or logs. |
| Clipboard | Only on explicit user action, with a warning that other apps can read it. |

Reference layout: `%LOCALAPPDATA%\Simentor\DesktopCredentials\{device-key.dpapi,session.dpapi,credentials.json}`.

Consequences to accept: losing the Windows profile or deleting the key file means the cached ciphertext is unrecoverable — the fix is unregister + re-enrol + re-issue, not key recovery. Copying the cache to another PC is useless (DPAPI is user-bound, and the ciphertext is sealed to the original keypair).

---

## 7. Device limit and lifecycle rules

- Max **2 active devices per Simentor user**. Unregister or admin revoke frees a slot immediately.
- Enrolment is auto-approved for verified `@bps.go.id`; no admin step.
- Enrolment grants **no** credentials. A super-admin must issue each one explicitly.
- On sign-out: delete the session only. Keep the device key and cache so already-downloaded credentials still work offline.
- On unregister: warn that future downloads stop but existing keys stay usable until upstream rotation. Ask before deleting local material; default is to keep it.

---

## 8. Required desktop UX

1. Login screen: email + password, BPS domain hint, clear error text per status code above.
2. Auto-enrol after login; show device id/label and slot usage (`1/2`).
3. Handle `409` with the exact message from §4.1 and a pointer to revoke an old device.
4. Credential list from cache, marked `Offline · locked` until decrypted; `Copy` enabled only after successful decrypt.
5. `Unregister device` action with the warning text in §7.
6. Never render ciphertext or key material in the UI, and never echo the plaintext back after issuance (desktop does not issue anyway).
7. Show the revocation caveat somewhere permanent, not just in a modal.

Reference implementation: `desktop/Simentor.Desktop/MainWindow.xaml{,.cs}`.

---

## 9. Offline behaviour

- Fetch requires network. Decrypt does not.
- On start: try to restore the session; if that fails (offline or expired), fall into ciphertext-only mode and say so plainly.
- Cached credentials remain readable offline after sign-out, unregister, revocation, or account deletion.

---

## 10. Acceptance checklist for the desktop team

- [ ] `tsc`/compiler clean; client starts with a readable error when `appsettings.json` is missing or not HTTPS.
- [ ] Sealed-box self-test passes on the target machines.
- [ ] Verified BPS login auto-enrols and shows slot usage.
- [ ] Third device gets `409` and the user-facing message.
- [ ] Issue from Simentor web → appears on device → decrypts → value usable.
- [ ] Ciphertext file copied to another PC cannot be decrypted.
- [ ] Unregister frees a slot; re-enrol works; old cached credentials still decrypt.
- [ ] Revoked credential disappears from the next fetch but already-cached copy still decrypts (documented, expected).
- [ ] No plaintext, token, or private key in logs, crash dumps, or `%TEMP%`.
- [ ] Release build signed (see §12).

---

## 11. Out of scope for the desktop team

- Issuing credentials, choosing owners/devices, pricing/entitlement logic — all web-side, super-admin only.
- Upstream service-key rotation — security/ops, see the runbook.
- Modifying Edge Functions or RLS. If the contract is wrong, raise it rather than working around it.

---

## 12. Open items to settle before release

1. **Deployment target**: hosted project ref + who runs `supabase functions deploy` and `migration up`.
2. **Code signing**: certificate, timestamping, and installer format (MSIX vs. Inno/NSIS) — not covered by this repo.
3. **Downstream integration**: the reference client only decrypts and copies. Which service consumes the credential, and does it need an API instead of clipboard?
4. **Concurrency proof**: the DB guard is atomic by construction (per-user advisory lock + partial unique index on active slot), but no two-session test has been executed. Worth one manual test with two PCs on the same account.
5. **`last_seen_at`**: currently set at registration only. Decide whether the client should ping a heartbeat action (would be a new `device-enroll` action) or leave it static.
6. **Bug reporting**: tickets go to the Simentor web repo owners; include the `device_id` and timestamp, never the credential value.

---

## 13. Verification status you can quote

Locally verified in this branch: TypeScript clean, full Vitest suite 136 files / 1049 tests passing, `dotnet build`/`publish` (`win-x64`, self-contained) clean, sealed-box round-trip test.

Not verified, environment blocked: migration execution, SQL test run, function deployment, hosted end-to-end. Treat §10 as the gate before telling anyone this is production-ready.
