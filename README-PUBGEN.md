# Pubgen: the double approach

One publication pipeline, two ways to drive DesignCraft — a standalone orchestrator for
everywhere, and a headless compiler for exact panel parity. Both ship together; pick per job.

```
                       ┌────────────────────────────────────────────┐
   Excel workbook ───► │  pos.app  (Publication Orchestrator Studio) │  GUI: import, metadata,
                       │  saves  project.pubgen (metadata + content) │  pages, slots, glossary
                       └───────────────┬────────────────────────────┘
                                       │
              ┌────────────────────────┴───────────────────────────┐
              ▼                                                    ▼
   APPROACH 1 — engine script                       APPROACH 2 — pubgen-cli
   (stock binaries, any machine)                    (one pubgen build, panel parity)
              │                                                    │
   build_batch_script() → fill.jsonl                pubgen-export → pubgen_project.json
   plain engine commands                            (version-1 manifest, profile by id)
   (frame.create, story.setText,                    pubgen-cli compile
    table.convertFromText, toc.generate)              spawns designcraft --control <free port>
              │                                      probes engine.commands for
              ▼                                      pubgen.compileProject, calls it,
   designcraft-cli script fill.jsonl                 kills the child
   --save out.designcraft                                        │
              ▼                                                  ▼
   out.designcraft + PDF                           out.designcraft + PDF
   (pos.app's own layout math —                     (the repo compiler: covers, front matter,
    known small drift vs the panel)                  TOC lists, running headers, pagination —
                                                    byte-for-byte what the panel produces)
```

## When to use which

| | Approach 1 — pos.app batch | Approach 2 — pubgen-cli |
|---|---|---|
| Needs | stock `designcraft.exe` + `designcraft-cli.exe` (the portable release) | **one** pubgen-enabled `designcraft.exe` (`cargo build -p designcraft --features pubgen`) |
| Output | Good, but pos.app's own frame math → small visual drift vs the panel | Exact parity with the PubGen panel compiler — the drift problem this solves |
| Where | Any machine, zero extra build | Anywhere you can ship the pubgen exe beside it |
| Input | `.pubgen` project (pos.app GUI) | `.pubgen` project + original chapter workbooks (`--workbook`), or a hand-written manifest |

## Shipping (Layout 1 — one designcraft.exe)

The pubgen feature is additive (every stock command still present, 763 vs 705 in the dev
builds), so one designcraft serves both approaches:

```
YourApp/
├── pos.exe                 standalone front-end (both approaches start here)
├── designcraft.exe         pubgen build — serves pos.app AND pubgen-cli
├── designcraft-cli.exe     stock is fine (Approach 1's script runner)
├── pubgen-cli.exe          Approach 2 compiler driver
└── pubgen-export.exe       .pubgen → pubgen_project.json converter
```

- `pubgen-cli` finds designcraft by trying candidates and keeping the **first that exposes
  `pubgen.compileProject`** (`PUBGEN_DESIGNCRAFT_EXE`, beside itself, dev `target/` dirs,
  `assets/designcraft-pubgen/`, `PATH`). A stock build is skipped, not an error.
- Ports never clash: pos.app uses fixed 7979; pubgen-cli binds a free port per spawn.

## Workflow, end to end (Approach 2)

```sh
# in pos.app: import the Master Excel, edit metadata/pages, save project.pubgen, then:
pubgen-export "C:/Projects/NTP/BRS_NTO.pubgen" --workbook "C:/Projects/NTP/ntp202610.xlsx"
pubgen-cli compile "C:/Projects/NTP/pubgen_project.json"
# → <stem>.designcraft + <stem>.pdf, same pages the panel would build
```

The manifest is **version 1** on purpose: profile named by id (`bps_dda`, `bps_brs`,
`academic_monograph`, `corporate_annual` — the four the compiler bundles), styling the panel
persists but pos.app does not stays at compiler defaults, so a compile reproduces the panel's
default build for that profile instead of a half-invented snapshot. Missing `profileId` is
inferred (BRS margins 18/18/20/20 or a release number → `bps_brs`); `--profile` overrides.

Details, schema, and the candidate-probe order: [`apps/pubgen-cli/README.md`](apps/pubgen-cli/README.md).
Panel compiler internals: [`docs/pubgen-guide.md`](docs/pubgen-guide.md) §10.

## Verified (pubgen.0.8.6)

| Run | Result |
|---|---|
| `pubgen-export` on `pubgen/samples/brs/BRS_NTO_202610.pubgen` | `bps_brs` inferred, manifest built, exit 0 |
| `pubgen-cli compile` (staged ship folder, foreign working dir) | 16 pages, 448 ms, `.designcraft` + `.pdf`, exit 0 |
| DDA single chapter (`BAB_01_2025.xlsx`) | 12 pages, 1.6 s |
| DDA two chapters, bilingual (`BAB_01` + `BAB_02`) | 20 pages, 1.4 s |
| `cargo test -p pubgen-cli` | 3/3 |

Scratch evidence: `scratch/pubgen-e2e/`, `scratch/pubgen-e2e-brs/`, `scratch/pubgen-e2e-dda/`,
`scratch/pubgen-ship/` (the Layout-1 staging copy).

## Version history

- **pubgen.0.8.6** — double approach formalized: `apps/pubgen-cli` crate (`pubgen-cli`,
  `pubgen-export`), exe-relative candidate probing for the portable layout, this document.
- **pubgen.0.8.5** — manifest version 2 recipe (`profileData`, `generateSettings`, `pagePlan`
  write-back) in the panel compiler; v1 manifests still compile byte-for-byte.
