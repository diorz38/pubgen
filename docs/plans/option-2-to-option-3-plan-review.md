# Strategic Review: Option 2 → Option 3 Migration Plan

**Document Under Review:** [`option-2-to-option-3-plan.md`](file:///e:/Github/pos_v3/option-2-to-option-3-plan.md)  
**Date:** November 2026  
**Context:** Evaluating the transition from **Option 2** (Pubgen integrated into DesignCraft workspace) to **Option 3** (decoupled publication product powered by DesignCraft engine), specifically analyzing the fate of the existing **`pos`** application and mitigating merge conflicts against active upstream **`designcraft`** development.

---

## 1. Executive Summary

The proposed plan is architecturally sound and targets the correct end-state: **treating `pubgen` as the canonical publication product while leveraging `designcraft` purely as a generic layout and rendering engine**.

Crucially, the plan avoids the catastrophic mistake of re-implementing text composition (Knuth–Plass), optical margin alignment, hyphenation, or PDF/X-4 export outside DesignCraft. However, executing this transition requires strict operational discipline in two specific areas:
1. **Managing the POS retirement cleanly** without dragging legacy UI patterns into the new codebase.
2. **Shielding Pubgen's evolving GUI from the high commit velocity of upstream DesignCraft**.

---

## 2. Impact on the `pos` Application (`/pos` folder)

The plan assigns a strictly bounded lifecycle to the existing `pos` application:

```
[ Active Application ] 
       ↓ (Phase 0)
[ Feature Freeze / Legacy Status ] 
       ↓ (Phases 1–2)
[ Generation Path Redirected ] 
       ↓ (Phases 3–5)
[ Capabilities Harvested ] 
       ↓ (Milestone 6)
[ Deprecated & Archived ]
```

### A. Feature Freeze (Immediate)
* Feature development on `pos` halts immediately.
* Only bug fixes directly unblocking migration are permitted.
* It ceases to be the forward-looking product and becomes a temporary reference donor.

### B. What Happens to the POS UI (`iced`)
* **The `iced` code is NOT migrated.**
  * `pos` uses `iced` (Elm-style architecture), whereas `pubgen`/`designcraft` uses `egui`.
  * The plan categorizes the POS `iced` UI under **Group C ("Redesign, not copy")**.
  * The monolithic UI patterns (such as giant action dispatchers in `actions.rs`) are discarded rather than ported.
* **The UI workflows and capabilities ARE migrated.**
  * The user experiences (BPS DDA/BRS profiles, metadata forms, content list cards, cover studio, dynamic TOC generator, vector charts, choropleth maps) are rebuilt natively in Pubgen's UI stack (`pubgen-ui` / `pubgen-studio`).

### C. What Happens to the Generation Engine
* The legacy ad-hoc POS script pipeline (`pos → designcraft-cli → batch JSONL script`) is replaced by the canonical manifest flow (`pubgen-export → pubgen-cli → pubgen.compileProject`).
* The legacy script generator remains solely as a read-only debug fallback until full feature parity is achieved.

### D. Final Retirement
* Once `pubgen` achieves full parity on all publication fixtures, a compatibility layer will ensure existing `.pubgen` project containers can still be opened.
* A deprecation notice is shown to remaining POS users.
* The `/pos` codebase is formally archived and removed from active builds.

---

## 3. Mitigating Merge Conflicts with Active Upstream `designcraft`

### The Problem: The "Hard Fork" Trap
Because `pubgen` is currently rooted in a fork of `designcraft`, heavily customizing ("radicalizing") the GUI inside existing DesignCraft UI files (`crates/ui-egui/src/panels/`, `apps/designcraft/src/main.rs`, etc.) will guarantee **continuous, painful merge conflicts** whenever upstream DesignCraft commits UI adjustments, canvas refactors, or new editor panels.

### The Solution: Strategic Decoupling Tactics

To allow radical innovation on the Pubgen GUI without suffering code conflicts, the following four architectural rules must be enforced:

```
┌─────────────────────────────────────────────────────────────┐
│                      UPSTREAM ZONE                          │
│  (Kept clean to allow automatic git pulls from upstream)    │
│                                                             │
│  apps/designcraft/                                          │
│  crates/engine, doc, compose, render, pdf                   │
└──────────────────────────────┬──────────────────────────────┘
                               │ (Consume as clean crate dependencies)
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                       PUBGEN ZONE                           │
│     (Isolated namespaces where upstream never touches)      │
│                                                             │
│  apps/pubgen-studio/      (Your independent GUI executable) │
│  crates/pubgen-ui/        (Your redesigned panels & views)  │
│  crates/pubgen-core/      (Headless compiler & domain logic)│
│  crates/pubgen-protocol/  (Manifests & control schemas)     │
└─────────────────────────────────────────────────────────────┘
```

#### Tactic 1: Create `apps/pubgen-studio`, Leave `apps/designcraft` Clean
* **Never radicalize `apps/designcraft/src/main.rs`.**
* Build a distinct binary target: `apps/pubgen-studio/`.
* When upstream updates the generic DesignCraft editor application, git merges apply cleanly because upstream never touches `apps/pubgen-studio/`.

#### Tactic 2: Isolate Pubgen UI in a Separate Crate (`crates/pubgen-ui`)
* Rather than scattering pubgen panels across `crates/ui-egui/src/panels/`, house all publication GUI screens inside a separate crate: `crates/pubgen-ui/` (or a strictly self-contained sub-crate).
* If pubgen must be toggleable inside DesignCraft during early phases, use a **single one-line feature flag** in `ui-egui/src/app.rs`:
  ```rust
  #[cfg(feature = "pubgen")]
  self.pubgen_panel.show(ctx, ui);
  ```
  This reduces the upstream conflict surface area in the shared UI code to almost zero.

#### Tactic 3: Rely on Engine Crates, Not Editor GUI Crates
* Upstream git churn is heavily concentrated in the desktop GUI layer (shortcuts, widget state, canvas interactions, toolbars).
* Lower-level engine crates (`crates/engine`, `crates/compose`, `crates/doc`, `crates/render`) change much less frequently and maintain formal type signatures.
* `pubgen-core` must depend only on these stable engine crates. It must **never depend on `ui-egui` or `eframe`**.

#### Tactic 4: The Adapter Boundary (`DesignCraftRuntime`)
* Implement the `DesignCraftRuntime` trait specified in Phase 4 of the plan.
* All calls to DesignCraft layout or rendering operations must route through this single adapter layer.
* When upstream refactors an engine method, you only update **one adapter implementation file**, rather than fixing compiler errors across dozens of publication UI screens.

#### Tactic 5: Protocol / Manifest Fallback (Maximum Isolation)
* If upstream development becomes extremely disruptive, `pubgen-studio` can execute compilation out-of-process via `pubgen-cli` or via loopback control (`127.0.0.1:7979`) using versioned `pubgen_project.json` manifests.
* Out-of-process execution completely decouples the GUI build toolchains.

---

## 4. The Dual-Mode Workspace & Container Architecture (Brainstorming Synthesis)

To provide an optimal user journey without sacrificing DesignCraft's layout capabilities, `pubgen-studio` implements a **Workspaces / Perspectives** paradigm:

### A. Dual Workspaces: Orchestrator vs. DesignCraft
```text
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│ [Pubgen Studio]    [ 📊 Orchestrator (Active) ]  [ 🎨 DesignCraft Layout (🔒/Active) ]   │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Top Menus dynamically switch based on active mode:                                      │
│ • Orchestrator Mode: File | Profiles | Ingest | Glossary | Covers | Preflight           │
│ • DesignCraft Mode:  File | Edit | Layout | Type | Object | View | Window               │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

1. **`StudioMode::Orchestrator`:**
   - Active on default launch.
   - Hosts all POS-migrated publication workflows: profile selection (BPS DDA/BRS), Excel sheet ingestion, metadata review, chapter list, cover studio, and TOC configuration.
   - Free of DTP canvas clutter while structuring data.
2. **`StudioMode::DesignCraft`:**
   - Unlocked when pages are generated (or when a `.designcraft` file is opened).
   - Hosts the full DesignCraft document canvas, vector tools, typography panels, swatches, and preflight overset inspector.
   - Allows fine manual adjustments and direct PDF/X-4 export.
3. **Dynamic Chrome & Navigation:**
   - Menu bar and tool panels switch completely according to active mode.
   - User can freely toggle back and forth between Orchestrator and DesignCraft modes once a document exists.

### B. "Recipe" (`.pubgen`) vs. "Artifact" (`.designcraft`)

```text
┌──────────────────────────────────────┐         Compile         ┌──────────────────────────────────────┐
│          *.pubgen (Recipe)           │ ──────────────────────> │       *.designcraft (Artifact)       │
│                                      │                         │                                      │
│ • ZIP container                      │                         │ • Native DesignCraft layout file     │
│ • Source data (Excel, CSV, MD)       │                         │ • Generated pages, frames, styles    │
│ • Metadata, glossary, assets         │                         │ • Manual layout tweaks by user       │
│ • Publication profile rules          │                         │ • Direct export to PDF / EPUB        │
└──────────────────────────────────────┘                         └──────────────────────────────────────┘
         Controlled by:                                                    Controlled by:
      📊 ORCHESTRATOR MODE                                              🎨 DESIGNCRAFT MODE
```

- **`*.pubgen` as Project Container:** Continues the POS format as a ZIP package containing `manifest.json`, `metadata.json`, `content/*.md`, `content/*.csv`, `assets/`, and `glossary.json`. Optionally bundles a cached `output/publication.designcraft`.
- **`*.designcraft` as Layout Document:** Houses all rendered frames and manual user styling. Layout tweaks stay inside `.designcraft` for now (no immediate reverse-sync to `.pubgen`).
- **Re-compilation Safety Guard:** When clicking "Compile" in Orchestrator, if an existing `.designcraft` layout exists with manual edits, present a 3-way guard dialog:
  - *Overwrite Existing*
  - *Save as New Version (v2)*
  - *Cancel*

### C. Dual File Association & Auto-Routing
The shell registers both extensions at the OS level:
- Opening a `*.pubgen` file -> boots into **Orchestrator Mode**, extracts project data, and unlocks DesignCraft if a compiled layout is cached.
- Opening a `*.designcraft` file -> boots directly into **DesignCraft Mode** with canvas ready for manual editing.

---

## 5. Key Strengths of the Plan

1. **Pragmatic Reuse:** Reuses DesignCraft's layout and typesetting strengths (Knuth-Plass, optical margins, hyphenation, PDF export) without reinventing them.
2. **Headless First (`pubgen-core`):** Forcing `pubgen-core` to be free of GUI dependencies ensures that publications can be compiled in CI/CD, batch scripts, or cloud workers without requiring a desktop window.
3. **Machine-Readable Contracts:** Replacing ad-hoc CLI stdout parsing with structured JSON reports (`status`, `warnings`, `errors`, `diagnostics`) eliminates fragile string scraping.
4. **Parity Testing Gate:** Requiring visual snapshot and golden fixture validation prevents regressions during the feature migration from POS.

---

## 6. Recommended Adjustments to the Plan

To strengthen execution against upstream conflicts, consider adding these explicit rules to the document:

| Item | Current Risk | Recommended Adjustment |
|---|---|---|
| **Upstream Branch Strategy** | Ad-hoc git pulls cause messy branch histories | Maintain a dedicated `upstream-main` branch that tracks official DesignCraft directly. Merge `upstream-main` into a `pubgen-integration` branch on a weekly schedule. |
| **GUI Location in Phase 3** | Plan suggests `crates/ui-egui/src/panels/pubgen/` | Move directly to an isolated `crates/pubgen-ui/` and `apps/pubgen-studio/` to avoid any modification to upstream's `crates/ui-egui/`. |
| **API Version Pinning** | Breaking changes in upstream engine could break pubgen builds | Pin DesignCraft engine dependencies using local path references with internal crate semver checks (`DesignCraftCompatibilityVersion`). |
| **Feature Deprecation Flags** | Users continue relying on POS out of habit | Add an explicit banner in POS during Phase 2 pointing users to the new Pubgen workflows. |

---

## 7. Conclusion

* **For the `pos` folder:** The path is clear—freeze, extract domain logic, discard the `iced` UI, redirect the compiler path, and archive the repository.
* **For upstream conflicts:** Radicalizing the publication GUI is completely viable **if and only if** you create a separate application entry (`pubgen-studio`) and separate crate (`pubgen-ui`), consuming DesignCraft's lower-level engine crates via an adapter rather than modifying DesignCraft's native UI panels.

