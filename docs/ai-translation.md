# AI Translation — how it works and how to use it

This page explains the PGS Orchestrator's Indonesian → English AI translation: what happens
inside a run, where results are stored, and the workflow from worksheet to bilingual
publication. The user-facing steps are also in `docs/pubgen-guide.md` §13; this page goes
deeper into the mechanics.

Translation is a **pre-compile stage**, not post-generation text rewriting. Units are
collected from the worksheet's cards before any page exists, translated once, stored in a
translation memory inside the `.pubgen` package, and then reused on every later generate —
so re-opening a project costs zero provider requests.

## Providers

Three online providers, configured in the Orchestrator's AI settings dialog:

| Provider | What you configure | Endpoint |
|---|---|---|
| **Groq** | API key + model (default `llama-3.3-70b-versatile`) | `https://api.groq.com/openai/v1` |
| **Custom** (any OpenAI-compatible API) | Base URL + API key + model id | your URL, e.g. `https://host/v1` |
| **Local AI** (Ollama, LM Studio) | Base URL + model id — no key | default `http://localhost:11434/v1` |

- The Groq rate-limit counter (`x-ratelimit-remaining`) is stored **only** for the Groq
  endpoint; custom and local endpoints never write it.
- Local AI works on plain HTTP because it stays on `localhost`.
- **TLS note:** the HTTP client trusts Mozilla's public root store only. An endpoint behind
  a private CA will fail; use plain HTTP on localhost, or a publicly-trusted certificate.
- The API key travels in the `Authorization` header only. It is never written into the
  translation memory, the `.pubgen` package, logs, or error messages, and the config
  struct's debug output redacts it.

Leave the provider on `offline` to disable the stage entirely.

## What a run does

Pressing **Translate Worksheet** runs this pipeline:

1. **Collect units.** Every card contributes its translatable text with a role — card
   titles, table headers and cells, notes, cover titles, narrative paragraphs. Each unit
   records its source text and role so the result can be written back to the right field.
2. **Skip what must not change.** Numbers, identifiers, dates, region proper nouns and
   person names are recognized by rule and never sent to the model. This is deliberate:
   statistical figures must survive translation byte-identical.
3. **Resolve local hits first.** Units already in the translation memory — and whole-string
   hits in the glossary — are reused without any network request. A summary reports how
   many units were reused.
4. **Mask.** For the misses, numbers and glossary terms are replaced with `[[0]]`, `[[1]]`,
   … placeholders (capped) before the text leaves the machine. The model translates prose
   around tokens it cannot corrupt, and the system prompt repeats the rule the mask enforces:
   keep every figure, ratio (`30:1`), percentage, range, date and unit exactly as written.
5. **Request.** The misses go to the provider as one background job. Strict servers are
   respected: the optional `reasoning_effort` field is only sent for `gpt-oss` models.
6. **Unmask and verify.** Placeholders are restored **from their source token**, so a figure
   cannot come back changed. A mangled, reordered or invented placeholder is repaired rather
   than dropping the paragraph: the known figures go back, an unknown `[[9]]` is removed and
   never shipped. Nothing is discarded for its numbers; a unit is only left out for an empty or
   fragmentary answer, and that is reported (`AI left N text(s) untranslated: ...`) instead of
   vanishing. A wrong item count from the model is an error, with item-by-item salvage for the
   rest.
7. **Enforce the glossary.** Official glossary wording (e.g. BPS terms) is applied
   deterministically after unmasking — the same substitution the masking already
   guaranteed, as a second, non-AI net.
8. **Write back as drafts.** Results are inserted into the translation memory with the
   origin `ai`. Existing entries with origin `reviewed` are **never overwritten** — your
   human decisions win over every later AI run.

The whole stage runs in-process; no external tool is spawned.

## The Glossary & Its Connection to AI Translation

Official statistical publications require strict consistency: standard institutional terms cannot be left to probabilistic language models. For example, Badan Pusat Statistik (BPS) establishes strict conventions such as *Kecamatan* → *Subdistrict*, *Kabupaten* → *Regency*, *PDRB* → *GRDP*, and *Luas Wilayah* → *Total Area*. An LLM translating freely might alternate between "district", "sub-district", "borough", or "gross regional domestic product" across different tables.

To solve this, the Orchestrator pairs the AI model with an authoritative, customizable **Glossary** (`bps_glossary.json`), forming a multi-layer safety net.

### How the Glossary Connects with AI Translation

The connection between the glossary and the translation engine operates across five complementary layers:

```
[Worksheet Unit]
       │
       ├─► [1. Whole-String Glossary Hit?] ───(Yes)───► [Save to TM as "imported"] ──► [Zero Network Cost]
       │                    │
       │                   (No)
       │                    ▼
       ├─► [2. Prompt Seeding] ───────────────► Injects official vocabulary into LLM system prompt
       │                    │
       ▼                    ▼
[3. Pre-Request Masking] ─────────────────────► Replaces terms & numbers with [[0]], [[1]] placeholders
       │
       ▼
[AI Model Translation]
       │
       ▼
[Unmask Placeholders]
       │
       ▼
[4. Deterministic Enforcement (glossary_fix)] ► Scans output and forcibly enforces official terms
       │
       ▼
[5. Number Census & Validation] ──────────────► Verifies exact numeric integrity
       │
       ▼
[Save to TM as "ai draft"]
```

1. **Layer 1: Deterministic Local Resolution (Zero Network Cost):**
   When an entire card title, table header, or label exactly matches an official glossary term (case-insensitively), it is resolved locally on your machine with zero latency and zero API cost. The result is immediately recorded into the translation memory with the origin `imported`.
2. **Layer 2: Prompt Seeding & Vocabulary Context:**
   When non-glossary prose and sentences must be sent to the AI model, active glossary pairs (`Indonesian = English`) are injected directly into the system prompt (up to the prompt token budget). This guides the LLM to choose official statistical phrasing in surrounding grammatical context.
3. **Layer 3: Pre-Request Token Masking (`[[n]]`):**
   Before text leaves the machine, any known glossary term embedded inside longer sentences (along with all numbers) is extracted and replaced with placeholder tokens like `[[0]]`, `[[1]]` (longest match wins). The LLM translates the syntax around the tokens without being able to modify or mistranslate the protected terms.
4. **Layer 4: Post-Response Deterministic Enforcement (`glossary_fix`):**
   Even if an AI model produces non-standard terminology or ignores prompt hints, the engine runs a deterministic regex-free word-boundary replacement algorithm (`pubgen_core::glossary_fix::enforce`) after unmasking. Any instance of an unapproved term is replaced with its official equivalent. The stage logs this explicitly: `"Glossary enforced X replacement(s) on AI output"`.
5. **Layer 5: AI-Assisted Glossary Harvesting ("Fill AI Drafts"):**
   The relationship is bidirectional. As new tables and worksheets are imported, unrecognised Indonesian terms are automatically harvested into the glossary. You can then ask the AI model to generate candidate English drafts for them in bulk. Once reviewed by a human editor, they become authoritative official terms.

### Managing the Glossary (Terminology Editor)

The application includes a dedicated glossary management interface:

- **Where to find it:** Open **Publication Tools ▸ AI Translate ▸ Setup…** (or click **AI Translator** in the PubGen panel header) and switch to the **Terminology** tab.
- **The Terminology Grid:**
  - **Indonesian**: The source terminology.
  - **AI English**: Candidate draft generated by the model for review.
  - **Official English**: The authoritative translation used by masking, prompt seeding, and deterministic enforcement.
  - **Seen**: How many times this term has been encountered in publication cards.
- **Available Actions:**
  - **Fill AI Drafts**: Asks the configured AI provider to supply English drafts for every term whose AI and Official fields are currently empty (up to 200 terms per click).
  - **Auto Checkbox**: When checked, newly harvested Indonesian terms from subsequent imports or generation runs are automatically sent for AI drafts.
  - **Import CSV…**: Merge an external two-column CSV or TSV file (`Indonesian,English`, delimiter auto-detected, header optional) into the glossary.
  - **Reset to bundled**: Reverts all terms back to the default bundled official BPS dictionary (`assets/pubgen/bps_glossary.json`), clearing any custom path override.
  - **Save…**: Saves the edited terms to a custom `bps_glossary.json` file. The file path is saved in your application preferences (`pubgenGlossaryPath`) and reloaded on startup.

## Translation Memory vs. Glossary: How They Differ

Both files are stored inside the `.pubgen` project container, but they serve distinct roles:

| Feature | Translation Memory (`translation_memory.json`) | Glossary (`glossary.json`) |
|---|---|---|
| **Scope** | Stores translated units for the specific publication (titles, table cells, notes, paragraphs). | Stores universal vocabulary terms and domain terminology (e.g. BPS nouns). |
| **Origin Tracking** | Tracks whether an entry is `ai` (draft), `reviewed` (human-approved), or `imported` (glossary hit). | Tracks `official` approved translation vs. candidate `ai` draft. |
| **Lifecycle** | Grows as worksheet cards are translated; capped at 100,000 entries. | Stays consistent across projects; can be imported/exported or reset. |
| **Override Rule** | `reviewed` entries are immutable and will never be overwritten by future AI runs. | `Official English` outranks all AI suggestions and is deterministically enforced. |

Because both files are bundled into the `.pubgen` zip container:
- Saving, closing, and reopening a project preserves all human approvals and glossary modifications.
- Re-running **Translate Worksheet** reuses every cached unit, resulting in 0 provider API requests.


## Rendering the English

On **Generate**, the worksheet's translation memory is seeded into the page pipeline. Two
flags govern what renders:

- `bilingual_output` — derived from the document language (any *bilingual* / *dwibahasa* /
  *Indonesian-English* value, or the DDA profile). It is re-derived at the last point
  before page generation, because profile applications reset it.
- `translate_with_ai` — set when bilingual output is on **and** the memory holds at least
  one entry. Without a memory there is nothing to pair, so pages stay single-language.

With both on: titles render ID + EN, table cells render the source with an italic English
pair, narratives keep their paragraph styles with English body text, and TOC cards offer a
*Pair Bilingual* option.

## App Navigation & User Workflow (Orchestrator)

The application provides a dedicated UI for AI configuration, worksheet translation, review, and generation. Here is how to navigate each step in the interface:

### 1. Configure the AI Provider

Before running a translation, configure and verify your AI backend:

- **Where to open:**
  - From the top menu bar: **Publication Tools ▸ AI Translate ▸ Setup…**
  - Or in the PubGen panel header band: Click the **AI Translator** button (which displays a checkmark `✓` when already configured).
- **In the "Translate with AI — setup" modal:**
  - Switch between the two dialog tabs:
    - **Setup**: Choose your provider radio button:
      - **Online — Groq (free account)**: Paste your API key from `console.groq.com`, click **Refresh models** to load models (defaults to `llama-3.3-70b-versatile`).
      - **Custom API (OpenAI-compatible)**: Enter the Base URL (including `http://` or `https://`), optional API key, and model ID. Click **Refresh models**.
      - **Local AI (Ollama, LM Studio…)**: Enter your local server Base URL (defaults to `http://localhost:11434/v1`), and model ID. No key is needed.
      - **Offline on this computer**: Inactive stub (NLLB ONNX not bundled).
    - **Terminology**: Search, inspect, edit, or reset official BPS glossary terms and manage AI drafts.
  - Click **Use AI**: The app sends a small test query (`["tes"]`) to verify connectivity and credentials, saves the configuration to `%APPDATA%\pubgen-studio\settings.json` (on Windows), and arms the provider.

### 2. Prepare the Worksheet & Bilingual Language

- **Import the workbook:** Click **Import Worksheet** or drag-and-drop an `.xlsx` file into the Orchestrator. Cards appear in the center **Struktur Halaman** (Page Structure) tab.
- **Enable bilingual output:**
  - In Publication Settings / Profile metadata, set **Document Language** to a bilingual value (e.g. `Indonesian-English` / `Dwibahasa`).
  - Or in the PubGen dock panel header, toggle the language button to **Dual Lang (Bilingual)**.
  - Setting bilingual mode unlocks the **Title EN** field across cards and prepares the renderer for English pairing.

### 3. Translate the Worksheet

- **Where to find the button:**
  - Navigate to the **Right Dock** of the Orchestrator workspace.
  - Select the 2nd tab: **Profile & Execution** (Indonesian: **Profil & Eksekusi**).
  - Directly beneath the Publication Profile box, locate the **Translate Worksheet** button.
- **Run the stage & Monitor Progress:**
  - Click **Translate Worksheet**.
  - While processing, the button transforms into an active **running color progress bar**:
    - Displays a filled progress background and an animated continuous shimmer/wave sweep.
    - Shows live progress numbers and percentages: `Menerjemahkan dengan AI… X/Y (Z%)`.
    - Tooltip indicates: *Translating with AI… Click to cancel*.
    - Clicking the button while running safely cancels the background translation job without losing progress.
  - Units already present in the translation memory or glossary resolve locally with zero network requests.
  - Once finished, the bottom status bar and the center **Log Eksekusi** (Execution Log) tab display the run summary:
    `[{time}] Translated {n} unit(s), reused {m}, skipped {k}, failed {f}.`

### 4. Review & Accept Translations

You can inspect and approve translations in two convenient places:

- **A. Translation Review Pane (Right Dock ▸ Terjemahan):**
  - In the Right Dock, select the 3rd tab: **Translation Review** (Indonesian: **Ulasan Terjemahan**).
  - **Grouped by Unit Role Category:** Drafts are cleanly organized into four primary collapsible groups:
    - **Page Title**: Cover Titles, Narrative Titles, Table Titles, Chart Titles, Map Titles, Divider Titles, and Card Titles.
    - **Paragraph Content**: Narrative Paragraph Bodies.
    - **Table Content**: Table Headers, Table Row Labels, and Table Notes.
    - **Other**: Headers & Footers, Table of Contents, Chart Series, Image Captions, and other miscellaneous items.
  - **Group Header Actions:**
    - Each group header shows the category name and item count (e.g. `[-] Table Content (14)`).
    - Click the category title to collapse/expand `[-]` / `[+]`.
    - **Accept All**: Bulk-promotes all drafts within that category to `Reviewed` (preserving any in-place text edits made by the user).
    - **Reject All**: Bulk-removes all drafts within that category so they can be retranslated in a later run.
  - **Individual Draft Rows:**
    - **Unit Role Tag**: Explicitly identifies the specific unit role next to the source text (e.g. `[Table Header]`, `[Cover Title]`, `[Table Note]`).
    - **Indonesian source text** (bold, read-only).
    - **English target text** (editable text field).
    - **Accept button**: Promotes the single target to `Reviewed`. If you edited the text, your edited version is saved as `Reviewed`.
    - **Reject button**: Removes the single draft so it can be retranslated or re-collected in a future run.
- **B. Card Editor (Center Area):**
  - Click or double-click any card in the **Struktur Halaman** tab to open the tabbed Card Edit modal.
  - Next to the **Title EN** field, check the status badge:
    - **AI draft** (with an inline **Accept** button).
    - **Reviewed** (locked in; will never be overwritten by subsequent AI translation runs).
    - Official glossary hits appear directly without a draft badge.

### 5. Generate the Bilingual Publication

- Return to the Right Dock's **Profile & Execution** (**Profil & Eksekusi**) tab.
- Choose your generation action:
  - **Generate Publication** (Deep Cyan button): Generates all pages with bilingual pairing into DesignCraft layout mode.
  - **Generate & Publish PDF** (Emerald Green button): Generates layout pages and automatically exports the final PDF.
- **Verify the output:**
  - Switch to the center **Pratinjau Visual / Live Preview** tab or switch to **DesignCraft** canvas mode via the top mode switcher to inspect the layout:
    - Table headers and cells render Indonesian with italic English underneath.
    - Divider, chapter, and chart titles display bilingual pairs.
    - Narrative bodies maintain paragraph formatting, heading styles, and English text.
    - Table of contents (TOC) entries link accurately to bilingual sections.

### 6. Save & Reopen

- Press `Ctrl+S` or click Save to store the `.pubgen` project.
- Both `translation_memory.json` and `glossary.json` are packaged inside the project archive.
- Reopening the project and running **Translate Worksheet** will report that units are reused, costing zero API tokens.


## User workflow (headless CLI)

```
pgs-cli translate <project.pubgen> --provider <groq|custom|local> \
    --base-url <url> --model <id> [--key-env <VARIABLE>] [--dry-run]
```

- The API key is read from the environment variable **named by** `--key-env` — never passed
  as an argument, so it never appears in argv, shell history or process listings.
- `--dry-run` reports the units without calling the provider or writing the package.
- Prints a JSON report: `units`, `translated`, `reused`, `skipped`, `failed`. Non-zero exit
  when any unit failed.
- Writes only the translation memory into the package; compile itself stays as before.

## What is deliberately never translated

Numbers and statistical figures (masked, restored from their token, and checked again by the
audit below), identifiers and codes, dates,
region proper nouns, person names, and the layout itself (frames, styles, templates).

## Checking the figures on the page

**Audit Publication** (`preflight.pubgen`) compares the figures of each dual-language review
spread: the Indonesian column's numbers, ratios (`30:1`), percentages and reformatting variants
(`1.234,56` ↔ `1,234.56`, `12,5%` ↔ `12.5%`) against the English column's, as a multiset — order
and separators are free, a lost, rounded or invented figure is a `numberStatsParity` warning on
that page. Run it before exporting; it catches a statistic the model changed after masking was
capped, or one you edited by hand.

## Known limits

- The BRS **direct-table** rendering branch stays single-language; bilingual pairing
  applies to the Orchestrator Generate path for bilingual profiles with a non-empty memory.
- Manifest-based compilation and `pgs-cli compile` never seed translations; compiled output
  is single-language.
- Header/footer lines are not collected yet.
- Private-CA HTTPS endpoints are unsupported (see TLS note above).
- The direction is Indonesian → English only.
