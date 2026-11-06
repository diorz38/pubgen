# Mini Plan: Orchestrator Excel ↔ PUBGEN

## Tujuan

Membangun pipeline headless dan UI-integrated untuk:

- Impor `.xlsx`/`.xlsm` menjadi workspace publikasi.
- Menyimpan workspace sebagai container `.pubgen` portable.
- Membuka `.pubgen` lintas mesin tanpa broken link.
- Mengekspor workspace kembali ke workbook Excel terstandar.
- Mengompilasi workspace menjadi `.designcraft` dan PDF melalui engine in-process.
- Menjaga kompatibilitas terukur dengan format POS, bukan klaim kompatibilitas tanpa fixture.

## Prinsip tetap

1. `pubgen-core` menjadi pemilik domain, schema, container, ingest, export, validation, dan diagnostics.
2. UI hanya adapter: pilih file, tampilkan state/diagnostics, panggil service/command.
3. Engine tidak bergantung pada egui.
4. Semua input file dianggap hostile: batas ukuran, batas entry, validasi path, checked arithmetic, no panic.
5. Container menjadi sumber kebenaran; materialized files hanya cache kerja dengan checksum.
6. Semua format memiliki versioning, migration policy, dan unsupported-feature diagnostics.
7. Setiap fase berakhir dengan `cargo xtask ci` atau subset gate yang terdokumentasi.

## Status awal yang perlu dihormati

- `crates/pubgen-core` sudah ada dan berisi manifest, profile, plan, compiler, report.
- Workspace Cargo sudah mendaftarkan `pubgen-core`.
- ZIP tersedia melalui workspace dependency.
- Parser Excel belum boleh diasumsikan tersedia di core; dependency dan layer harus diverifikasi.
- UI PubGen lama berada di `crates/ui-egui/src/panels/pubgen/`.
- POS menjadi referensi perilaku/schema, bukan sumber kompatibilitas otomatis.

---

## Phase 0 — Contract freeze dan inventory

**Hasil:** baseline teknis yang dapat diuji.

### Pekerjaan

- Inventaris API/state lama dan tipe POS yang perlu dipetakan.
- Tetapkan schema `.pubgen` v1:
  - `pubgen.json`: format version, project identity, template presence, slots.
  - `metadata.json`: profile, layout, language, publication metadata, cards.
  - `glossary.json`: glossary version dan entries.
  - `csv/`, `md/`, `assets/`, optional `template.designcraft`.
- Tetapkan canonical JSON rules: field required/optional, unknown-field policy, UTF-8, stable ordering bila diperlukan.
- Tetapkan `PublicationCard`, `CardKind`, `CardScope`, source URI internal, asset reference, chart metadata.
- Tetapkan error taxonomy: invalid archive, unsupported version, missing entry, invalid path, size limit, schema error, Excel error, export error.
- Dokumentasikan batas kompatibilitas POS: compatible, readable-with-warning, incompatible.

### Gate

- Schema document disetujui secara internal.
- Test dapat membuat minimal manifest/card dan menolaknya saat field invalid.
- Tidak ada dependency UI di model domain.

---

## Phase 1 — Container `.pubgen` v1

**Hasil:** save/open portable, atomic, hardened.

### Pekerjaan

- Tambah modul container di `crates/pubgen-core`.
- Implement `save_pubgen`:
  - tulis sibling temp file dengan nama aman;
  - serialize manifest/metadata/glossary;
  - tulis content dan assets dari memory/state;
  - reject duplicate internal paths;
  - finish ZIP sebelum replace target;
  - gunakan atomic replacement yang sesuai platform.
- Implement `open_pubgen`:
  - validasi ZIP dan entry count/expanded-size limits;
  - reject absolute paths, `..`, path separator anomali, duplicate entries;
  - validasi required entries dan format version;
  - deserialize lalu validate cross-references;
  - materialize cache memakai checksum dan safe directory.
- Pisahkan `ProjectContent` embedded dari `MaterializedWorkspace` cache.
- Tambah checksum per content entry dan cache invalidation.
- Tambah migration scaffold untuk `format_version` future/older.

### Test minimum

- Round-trip minimal, rich project, empty optional sections.
- Corrupt ZIP, truncated ZIP, duplicate paths, traversal path, oversized entry.
- Interrupted save tidak merusak target lama.
- Reopen menghasilkan state identik dan cache tidak stale.
- Windows path behavior.

### Gate

`cargo test -p pubgen-core`; clippy deny rules; fixture archive inspection; `cargo xtask layers`.

---

## Phase 2 — Excel ingest

**Hasil:** workbook menjadi normalized intermediate model, tanpa UI coupling.

### Pekerjaan

- Pilih dan pin reader crate yang lolos wasm/layer/license review; gunakan `calamine` bila kompatibel dengan target build.
- Tambah `excel::ingest` dengan pipeline:
  1. open workbook dengan input limits;
  2. enumerate sheets safely;
  3. normalize cell values/types;
  4. detect `COVER`/`COV`/`SAMPUL`;
  5. parse metadata aliases dan diagnostics untuk duplicate/unknown keys;
  6. detect profile BRS/DDA/unknown;
  7. classify sheets menjadi cards;
  8. convert tables/charts/narratives menjadi internal content;
  9. produce `IngestReport` plus `WorkspaceDraft`.
- Definisikan deterministic rules untuk:
  - merged/stacked headers;
  - formulas versus cached values;
  - dates, booleans, blanks, numeric precision;
  - sheet names dan invalid filename characters;
  - chart metadata yang tidak didukung;
  - narrative markdown extraction.
- Jangan langsung menulis `.pubgen` selama parsing; gunakan draft lalu validation.

### Test minimum

- Synthetic BRS workbook.
- Synthetic DDA workbook.
- COVER aliases and missing/duplicate fields.
- merged headers, empty rows, numeric precision, unicode, malformed metadata.
- Unknown profile produces actionable diagnostics, not guessed destructive output.
- Fuzz/property tests untuk sheet names/cell data bila feasible.

### Gate

Golden ingest fixtures stable; no panic on malformed workbook; wasm/build compatibility checked.

---

## Phase 3 — Workspace normalization and validation

**Hasil:** satu model workspace dipakai save, UI, compile, export.

### Pekerjaan

- Map `WorkspaceDraft` ke `OrchestratorWorkspaceState`/core equivalent.
- Implement validation:
  - card source exists;
  - card kind/scope combination valid;
  - page order deterministic;
  - profile settings valid;
  - asset references resolve inside project;
  - template and slot references valid;
  - no external absolute paths in portable state.
- Implement diagnostics with severity, source location, stable code, remediation text.
- Implement preflight command/service independent of UI.
- Add backward migration from legacy loose-file state where safe; otherwise explicit import warning.

### Gate

Invalid workspace cannot compile/save silently; diagnostics renderable by UI and serializable for CLI/control channel.

---

## Phase 4 — Engine compilation integration

**Hasil:** workspace menghasilkan document/PDF melalui in-process engine.

### Pekerjaan

- Adapt existing `pubgen-core` compiler/plan to consume normalized cards.
- Define command plan for:
  - new document/layout;
  - cover, divider, TOC;
  - table/chart/map/narrative cards;
  - autoflow;
  - header/footer;
  - preflight and export.
- Keep execution transactional where possible: validate plan before mutation, return structured failure.
- Route all user-visible actions through engine commands.
- Reuse existing PubGen generation behavior, then move shared logic from UI into core incrementally.
- Add cancellation/progress hooks without storing transient execution log as project state.

### Test minimum

- Plan snapshot tests.
- Compile minimal BRS/DDA workspaces.
- Missing source, overset, missing asset, unsupported card diagnostics.
- PDF/designcraft output smoke tests and no-panic regression tests.

### Gate

Existing PubGen generation remains green; new path matches expected page/card order.

---

## Phase 5 — Excel export and round-trip policy

**Hasil:** deterministic `.xlsx` export with explicit loss rules.

### Pekerjaan

- Select/pin XLSX writer crate after license, wasm, size, and feature review.
- Implement `excel::export` from normalized workspace, not UI state.
- Write deterministic sheets:
  - `COVER` key-value metadata;
  - `POINT` summary items;
  - `T*` table cards;
  - `C*` chart data/series;
  - optional narrative sheets with documented encoding.
- Preserve numeric values as numeric where possible; never silently stringify precision-sensitive data.
- Define unsupported/lossy export diagnostics for style, formula, chart, image, map, and layout attributes.
- Sanitize sheet names and resolve collisions deterministically.
- Add export options: profile, language, cached values/formulas policy, include assets.

### Test minimum

- Workspace → XLSX → ingest semantic equivalence.
- Stable sheet ordering and names.
- Numeric/date/unicode/merged-header fixtures.
- Unsupported features produce warnings/errors according to policy.
- Output opens with an independent XLSX reader.

### Gate

Round-trip means semantic equivalence, not binary equality; contract documents accepted lossiness.

---

## Phase 6 — Orchestrator UI integration

**Hasil:** 3-panel studio workflow replacing fragmented modal flow.

### Pekerjaan

- Add welcome actions: Open `.pubgen`, Import XLSX, New workspace.
- Add studio panels:
  - project metadata/profile;
  - ordered publication cards;
  - source/assets/diagnostics.
- Wire Save/Open/Import/Export/Compile through core service.
- Keep file dialogs and egui state in UI crate only.
- Show progress, warnings, unsupported content, and recovery errors.
- Preserve old PubGen workflow behind compatibility path until parity gate passes.
- Add UI tests for empty, imported, dirty, invalid, and compile-failed states.

### Gate

No direct ZIP/XML/Excel parsing in UI; actions remain reachable through command/control surfaces where applicable.

---

## Phase 7 — POS compatibility, migration, and hardening

**Hasil:** compatibility claim backed by evidence.

### Pekerjaan

- Obtain/create representative POS fixtures without committing proprietary or large binaries.
- Build compatibility matrix by format version and feature.
- Verify open/save behavior against POS implementation where legally and technically allowed.
- Add migration tools for legacy loose files and old `.pubgen` versions.
- Add resource limits, cancellation, temp cleanup, crash recovery, and atomic-save recovery.
- Audit secrets/path leakage and external references.
- Run `cargo xtask ci`, wasm, layers, assets, and focused integration suites.

### Exit criteria

- Portable project opens on another machine with no broken internal links.
- Malformed archives/workbooks return errors, never panic.
- BRS/DDA import → save → reopen → compile → PDF passes fixtures.
- Exported XLSX re-ingests with documented semantic equivalence.
- POS compatibility matrix published with unsupported cases explicit.
- Legacy path remains usable or migration error is actionable.

---

## Dependency order

```text
Phase 0
  -> Phase 1
  -> Phase 2
  -> Phase 3
  -> Phase 4
  -> Phase 5
  -> Phase 6
  -> Phase 7
```

Phase 1 and Phase 2 can run in parallel after Phase 0. Phase 5 starts after normalized model stabilizes. Phase 6 starts after core APIs and diagnostics exist. Phase 7 requires all prior gates.

## Suggested delivery slices

1. v0: schema + minimal container save/open.
2. v1: BRS/DDA ingest to cards, no UI export.
3. v2: compile imported workspace to DesignCraft/PDF.
4. v3: Orchestrator UI and legacy migration.
5. v4: XLSX export and semantic round-trip.
6. v5: POS compatibility matrix and hardening.

## First implementation task

Start Phase 0 with schema types and validation tests in `crates/pubgen-core`, then add container APIs. Do not begin UI work or XLSX export before the normalized model and error contract are stable.
