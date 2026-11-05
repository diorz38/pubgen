# Pubgen: Publication Orchestrator & Compiler Architecture

**Pubgen** is the canonical automated publication pipeline built on top of the native **DesignCraft** page-layout engine. It transforms raw statistical datasets (BPS BRS, BPS DDA, corporate annual reports, academic monographs) into production-ready layouts and PDFs.

The codebase provides two primary interaction models sharing the exact same layout compiler:
1. **Interactive Dual-Mode GUI (`pubgen-studio`)**: A unified application featuring the **Publication Orchestrator** workspace and the full **DesignCraft Layout** canvas.
2. **Headless Compilation Pipeline (`pubgen-cli` & `pubgen-export`)**: Scriptable, deterministic CLI tools for CI/CD, batch processing, and server-side compilation with 100% panel parity.

```
                      ┌────────────────────────────────────────────────────────┐
                      │    pubgen-studio  (Dual-Mode Publication Studio)       │
                      │  [ 📊 Orchestrator Mode ]   [ 🎨 DesignCraft Layout ]  │
                      └──────────────────────────┬─────────────────────────────┘
                                                 │
                 ┌───────────────────────────────┴───────────────────────────────┐
                 ▼                                                               ▼
      MODE 1 — Interactive Studio                                     MODE 2 — Headless CLI
     (Dual Workspace in pubgen-studio)                               (pubgen-cli & pubgen-export)
                 │                                                               │
  ┌──────────────┴──────────────┐                                                │
  │ 📊 ORCHESTRATOR WORKSPACE   │                                                │
  │ • Center: Live Preview,     │                                                │
  │   Page Cards, QA, Log       │                                                │
  │ • Right Dock (InDesign Look)│                                 pubgen-export project.pubgen
  │   - Tab 1: Sumber & Konten  │                                        │
  │   - Tab 2: Profil & Eksekusi│                                        ▼
  │   - Collapse/Expand Icons   │                                 pubgen_project.json (v1/v2)
  └──────────────┬──────────────┘                                        │
                 │ [🚀 Generate / Compile]                               ▼
                 ▼                                                pubgen-cli compile
  ┌─────────────────────────────┐                                 (spawns engine, calls
  │ 🎨 DESIGNCRAFT LAYOUT MODE  │                                  pubgen.compileProject)
  │ Full native InDesign parity:│                                        │
  │ canvas, dock, rulers, tools │                                        ▼
  └──────────────┬──────────────┘                                 out.designcraft + PDF
                 │ [File › Export]                               (exact byte-for-byte panel parity)
                 ▼
          out.designcraft + PDF
```

---

## The Applications & Binaries

| Binary / Crate | Role | Primary Workflow |
|---|---|---|
| **`pubgen-studio`** (`apps/pubgen-studio`) | **Unified Dual-Mode GUI** | Interactive publication ingestion, card sequencing, metadata editing, live QA, and direct switching to native DesignCraft canvas. |
| **`pubgen-cli`** (`apps/pubgen-cli`) | **Headless Compiler CLI** | Automation driver that probes and spawns `designcraft` with the `pubgen` feature on an ephemeral port to compile manifests. |
| **`pubgen-export`** (`apps/pubgen-cli`) | **Manifest Exporter** | Converts `.pubgen` container files and Excel workbooks into normalized `pubgen_project.json` manifests. |
| **`designcraft`** (`apps/designcraft --features pubgen`) | **Core Layout Engine** | Adobe InDesign-class layout engine exposing native `pubgen` commands and UI dialogs (covers, charts, tables, TOC). |
| **`designcraft-cli`** (`apps/designcraft-cli`) | **Engine Script Runner** | Headless runner for executing raw engine scripts and command streams. |

---

## Orchestrator Workspace Architecture

The **Orchestrator Workspace** inside `pubgen-studio` follows an ergonomic, high-density layout designed for efficient publication assembly:

```
┌───────────────────────────────────────────────────┬────────────────────────────────┐
│  Center Panel: Eksekusi & Verifikasi              │ Right Dock: InDesign Parity    │
│  (Wide primary workspace, responsive flex)        │ (Width: 320 pt / Collapsed: 38)│
├───────────────────────────────────────────────────┼──┬─────────────────────────────┤
│ • Header/Footer Live Configuration & Geometry     │» │ Tab 1: Sumber & Konten      │
│ • QA Preflight Audit Status & Diagnostics         │  │ Tab 2: Profil & Eksekusi    │
│ • Tab Strip:                                      ├──┴─────────────────────────────┤
│   - [Struktur Halaman]: Page layout cards         │ [Tab 1 Active]                 │
│   - [Visual Live Preview]: Real-time page proof   │ • Pohon Konten (CSV / MD tree) │
│   - [Log Eksekusi]: Real-time compilation stream  │ • Import buttons (Word/Excel)  │
│                                                   │ • Publication Metadata Editor  │
│                                                   │ [Tab 2 Active]                 │
│                                                   │ • Publication Profile (BRS/DDA)│
│                                                   │ • ⚡ Generate & 🚀 Publish      │
│                                                   │ • Layout Engine Status         │
│                                                   │ • Slot Character Budget Meter  │
└───────────────────────────────────────────────────┴────────────────────────────────┘
```

### 1. Primary Center Canvas ("Eksekusi & Verifikasi")
* **Responsive Breadth:** Expands across all available width between the left edge and the right dock (`center_w = (total_width - right_w).max(300.0)`).
* **Workspace Tabs:**
  * **Struktur Halaman:** Card sequence representing publication spreads and sections (covers, narrative blocks, charts, tables) with drag-and-drop or modal editing.
  * **Visual Live Preview:** Live layout rendering with instant pagination verification.
  * **Log Eksekusi:** Detailed step-by-step diagnostic logging of compilation, slot resolution, and asset ingestion.

### 2. InDesign-Parity Right Dock
Implements Adobe InDesign / DesignCraft native docking principles ([`crates/ui-egui/src/panels/pubgen/workspace.rs`](crates/ui-egui/src/panels/pubgen/workspace.rs)):
* **Tab 1: "Sumber & Konten" (`OrchestratorRightTab::SumberKonten`):**
  * Hierarchical content tree tracking all imported CSV tables and Markdown narratives.
  * Ingestion action triggers (`+ Impor Sumber`, `📊 Impor Excel`, `🎨 Impor Meta`).
  * Live publication metadata form (Title, Subtitle, Release No., Date, Agency).
* **Tab 2: "Profil & Eksekusi" (`OrchestratorRightTab::ProfilEksekusi`):**
  * Publication Profile switcher (`BPS Berita Resmi Statistik (BRS)` vs `BPS Daerah Dalam Angka (DDA)`).
  * Page geometry specifications (Margins: T:20.2 B:20.0 L:19.0 R:15.0 mm).
  * Action buttons:
    * **`⚡ Generate` (Cyan `#0E7490`):** Compiles layout pages into native `.designcraft`.
    * **`🚀 Generate & Publish` (Emerald `#166534`):** Compiles and exports directly to ready-to-release PDF.
  * **Engine Integration Status:** Displays live engine connectivity (`● Terintegrasi (DesignCraft Native OK)`).
  * **Slot Terdeteksi:** Character budget meter comparing text against frame limits (`● OK` / `● Over` with live character counts).
* **Collapse to Icons & Expand:**
  * **Expanded Mode:** Full 320 pt dock with tab strip (`tab_height = 27 pt`), active tab accent indicators (`t.accent`), and collapse button (`»`).
  * **Collapsed Mode:** Collapses into a 38 pt vertical strip (`ICON_WIDTH`) showing the expand button (`«`) and vertical icon buttons (`panel-articles` for Tab 1, `panel-properties` for Tab 2). Clicking either icon immediately switches to that tab and expands the dock.

---

## Dual-Mode Switching & Safety Guard

`pubgen-studio` allows bidirectional toggling between modes via the top bar pills:
* **Switch to DesignCraft Layout:** Inspect and manually edit the compiled pages on the high-fidelity vector canvas.
* **Re-compilation Safety Guard:** If manual layout edits were made in DesignCraft mode, clicking "Generate" or "Compile" in Orchestrator mode triggers a 3-way guard dialog:
  1. *Overwrite Layout*: Recompile and discard manual changes.
  2. *Preserve Custom Layout*: Keep existing canvas modifications.
  3. *Save As New Revision*: Branch into a separate `.designcraft` file.

---

## Headless CLI Workflows (pubgen-cli)

For headless build environments and automated data pipelines:

```sh
# 1. Export container to manifest (if starting from a .pubgen project)
pubgen-export "C:/Projects/NTP/BRS_NTO.pubgen" --workbook "C:/Projects/NTP/ntp202610.xlsx"

# 2. Compile publication manifest headlessly
pubgen-cli compile "C:/Projects/NTP/pubgen_project.json"
# → Produces <stem>.designcraft and <stem>.pdf with exact panel parity
```

### Candidate Probing
`pubgen-cli` locates `designcraft` automatically by testing candidate binaries in order:
1. `PUBGEN_DESIGNCRAFT_EXE` environment variable
2. Sibling directory binary (`./designcraft.exe`)
3. Cargo dev target directories (`target/debug/`, `target/release/`)
4. Staged assets directory (`assets/designcraft-pubgen/`)
5. System `PATH`

The driver probes candidate binaries with `--control` and inspects `engine.commands` for `pubgen.compileProject`. If an engine build lacks the pubgen feature, `pubgen-cli` cleanly skips it and continues to the next candidate. Ports never clash because `pubgen-cli` binds an ephemeral free port per invocation.

---

## Shipping Layout (Portable Distribution)

```
PubgenStudio/
├── pubgen-studio.exe       Unified dual-mode GUI application
├── designcraft.exe         Pubgen-enabled core engine (serves CLI & studio)
├── designcraft-cli.exe     Stock engine script runner
├── pubgen-cli.exe          Headless compiler driver
└── pubgen-export.exe       Project container to manifest exporter
```

---

## Manifest Contract (`pubgen_project.json`)

* **Version 1 (`"version": 1`)**: Standard manifest defining publication profile by ID (`bps_dda`, `bps_brs`, `academic_monograph`, `corporate_annual`), metadata, document geometry, chapter list, and TOC config.
* **Version 2 (`"version": 2`)**: Extensible manifest including `profileData`, `generateSettings`, and round-trip `pagePlan` overrides.

---

## Version History

- **pubgen.0.8.6+** — Unified `pubgen-studio` application with dual-mode shell (`Orchestrator` ↔ `DesignCraft`), wide center canvas, and InDesign-parity Right Panel Dock (tabbed "Sumber & Konten" / "Profil & Eksekusi", collapse to icons & expand).
- **pubgen.0.8.6** — Double approach formalized: `apps/pubgen-cli` crate (`pubgen-cli`, `pubgen-export`), candidate probing for portable distribution.
- **pubgen.0.8.5** — Manifest version 2 recipe (`profileData`, `generateSettings`, `pagePlan` write-back) in the panel compiler; v1 manifests compile with full backward compatibility.
