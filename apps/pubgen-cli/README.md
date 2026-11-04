# pubgen-cli

Headless publication compiler for [DesignCraft](../..) publications. Two binaries, zero engine
code: the compiler itself lives in `crates/ui-egui/src/panels/pubgen` and runs inside a
pubgen-enabled `designcraft` process; these binaries only drive it over the control channel.

```
pubgen-cli compile <MANIFEST> [--pdf OUT.pdf]   # compile pubgen_project.json -> .designcraft (+PDF)
pubgen-cli info <MANIFEST>                      # summary, no build
pubgen-cli validate <MANIFEST>                  # chapters resolve, version-2 blocks present

pubgen-export <PROJECT.pubgen> [--out MANIFEST.json] [--workbook FILE.xlsx]... [--profile ID]
                                              # pos.app workspace -> pubgen_project.json
```

## Why this shape

The original `designcraft-cli` (no pubgen feature) cannot compile a publication: the compiler
lives in the UI crate and the feature is `#[cfg]`-gated. `pubgen-cli` does not port that code —
it spawns a pubgen-enabled `designcraft` with `--control <port>` (a free localhost port), checks
`engine.commands` exposes `pubgen.compileProject`, and calls it via
`engine.execute {"command":"pubgen.compileProject","params":{"path":…,"pdf"?…}}`. Those are the
only parameters the command accepts; `sync` defaults to true, so the reply arrives when the
document and PDF are on disk. The child is killed on exit.

## The one requirement

A pubgen-enabled designcraft executable. Build it once:

```sh
cargo build --release -p designcraft --features pubgen
```

`pubgen-cli` tries candidates in order and uses the **first that answers `engine.commands`
with `pubgen.compileProject`** (a stock build is skipped, not an error):

1. `PUBGEN_DESIGNCRAFT_EXE` environment variable
2. beside `pubgen-cli.exe` itself — the portable layout below, works from any launch folder
3. `target/debug/designcraft(.exe)` then `target/release/…` (dev checkout; debug first —
   that's where the freshly built feature binary lives)
4. `assets/designcraft-pubgen/designcraft(.exe)` relative to the working directory
5. `PATH`

Only when every candidate is stock does it stop, naming each one it tried.

## Shipping (Layout 1 — one designcraft.exe)

The pubgen feature is additive (every stock command is still there), so ship **one** designcraft
— the pubgen build — and the whole toolset works off it:

```
YourApp/
├── pos.exe                 standalone front-end (GUI, Excel import, batch fill)
├── designcraft.exe         pubgen build (serves pos.app AND pubgen-cli)
├── designcraft-cli.exe     stock is fine (pos.app's script path; identical engine commands)
├── pubgen-cli.exe
└── pubgen-export.exe
```

`pubgen-cli compile` spawns its designcraft on a **free** localhost port (pos.app's fixed 7979
is never touched), probes it, compiles, kills it — both can run at once. Verified staging copy:
`scratch/pubgen-ship/` (launched from an unrelated working directory → found the exe beside
itself → 16 pages, 448 ms).

## Workflow: from the pos.app (Publication Orchestrator Studio)

```
pos.app                    pubgen-export                 pubgen-cli                designcraft (pubgen)
  save project.pubgen --> pubgen_project.json -------> compile -------------> .designcraft + .pdf
  (metadata, pages,       (--workbook per chapter)     (spawns, drives,
   slots, glossary)                                    kills the child)
```

The `.pubgen` zip carries converted csv/md, not the chapter workbooks the compiler needs, so
pass the original Excel files:

```sh
pubgen-export "C:/Projects/NTP/BRS_NTO.pubgen" --workbook "C:/Projects/NTP/ntp202610.xlsx" --profile bps_brs
pubgen-cli compile "C:/Projects/NTP/pubgen_project.json"
```

The manifest is **version 1** on purpose: the profile is named by id (`bps_dda`, `bps_brs`,
`academic_monograph`, `corporate_annual` — the four the compiler bundles), and styling pos.app
does not persist (table style, header/footer, cover theme) stays at the compiler's profile
defaults. Version 1 compiles exactly like the reference panel's default build for that profile;
a half-remembered `generateSettings` snapshot would drift instead.

## Manifest schema (what pubgen-export writes)

```jsonc
{
  "version": 1,
  "publicationTitle": "…",            // metadata.title, else the project file stem
  "bilingual": false,                 // metadata.lang == "IdEn"
  "profile": "bps_dda",               // one of the four built-ins (or inferred: --profile)
  "output": { "document": "<stem>.designcraft", "pdf": "<stem>.pdf" },   // relative to the manifest
  "document": {
    "preset": "A4",                   // "Letter" when metadata.paper says so
    "facingPages": true,              // metadata.layout == "Double"
    "bleedMm": 3,
    "marginsMm": { "top": .., "bottom": .., "inside": metadata.margins.left, "outside": metadata.margins.right }
  },
  "chapters": [ { "chapter": 1, "path": "ABS/OLUTE/BAB_01.xlsx", "title": "BAB 01 2025" } ],
  "toc": { "lists": ["contents", "tables"?] },   // tables/charts only when such pages exist
  "build": { "keepIntermediate": true, "exportPdf": true }
}
```

Everything else (cover discovery, pagination, header/footer preset, front matter) keeps the
compiler defaults documented in `docs/pubgen-guide.md` §10. Missing `profileId` is inferred:
BRS margins (18/18/20/20) or a release number mean `bps_brs`, otherwise `bps_dda` — pass
`--profile` to be sure.

## Verified end-to-end

`scratch/pubgen-e2e/` holds the smoke run against `pubgen/samples/SINGLE_LANG/BAB_01_2025.xlsx`
(profile `bps_dda`, one chapter): `pubgen.compileProject` returned 12 pages, 1 table, 3
narrative pages, and wrote `e2e.designcraft` + `e2e.pdf` in 1.6 s through the same
spawn → probe → `engine.execute` protocol this CLI implements.

## Building here

```sh
cargo build --release -p pubgen-cli && cargo test -p pubgen-cli
```

Std-only: `serde`, `serde_json`, `zip` (workspace deps); no clap/tokio. `pubgen-cli compile`
needs a window-capable session (the child is the real app); `info`/`validate`/`pubgen-export`
are pure file operations.

`ponytail:` when pos.app starts persisting its panel styling (table style, header/footer,
cover theme) into the project, extend `pubgen-export` to emit a version-2 manifest with
`generateSettings` + `profileData` — the compiler already honours both when present.
