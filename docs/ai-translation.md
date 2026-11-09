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
   around tokens it cannot corrupt.
5. **Request.** The misses go to the provider as one background job. Strict servers are
   respected: the optional `reasoning_effort` field is only sent for `gpt-oss` models.
6. **Unmask and verify.** Placeholders are restored; a **number census** then rejects any
   unit whose numbers do not match the source exactly. A wrong item count from the model is
   an error, with item-by-item salvage for the rest.
7. **Enforce the glossary.** Official glossary wording (e.g. BPS terms) is applied
   deterministically after unmasking — the same substitution the masking already
   guaranteed, as a second, non-AI net.
8. **Write back as drafts.** Results are inserted into the translation memory with the
   origin `ai`. Existing entries with origin `reviewed` are **never overwritten** — your
   human decisions win over every later AI run.

The whole stage runs in-process; no external tool is spawned.

## Translation memory and glossary

Both live inside the `.pubgen` package, so they travel with the project:

- `translation_memory.json` — one entry per source string: source, target, role, origin
  (`ai` / `reviewed` / `imported`), and a timestamp. Capped at 100,000 entries; a package
  over the cap keeps the first 100,000 when opened.
- `glossary.json` — the persisted source→target glossary. It is separate from the per-run
  masking list so finishing a run never wipes the saved glossary.

Because the memory is inside the package: save, close, reopen → **Translate Worksheet**
reports every unit reused and makes zero provider requests. Edit a reviewed entry, and a
later AI run leaves it alone.

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

## User workflow (Orchestrator)

1. **Import** the worksheet (`.xlsx`) — cards appear.
2. **Set the document language** to a bilingual value, e.g. `Indonesian-English`
   (Publication settings).
3. **Configure a provider** in AI settings: pick Groq + paste key + model, or Custom +
   base URL + key + model, or Local AI + your Ollama/LM Studio URL + model. Test that the
   model list loads.
4. **Translate Worksheet** — the stage plans, resolves memory/glossary hits, then sends
   only the misses. A background job runs; the status bar and execution log report
   translated / reused / skipped / failed counts. If everything resolves locally, no
   request leaves the machine.
5. **Review** — the review pane lists results as *AI draft*. Fix wording, then **Accept**
   to store the entry as `reviewed`. The card editor's Title EN field shows a badge
   (AI draft / Reviewed) with an inline Accept button.
6. **Generate** — pages render bilingual from the memory.
7. **Save** the `.pubgen` — memory and glossary are inside; reopening reuses everything.
8. **Iterate** — editing a worksheet title creates a new unit; Translate Worksheet sends
   only it. Reviewed entries are untouched.

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

Numbers and statistical figures (masked and census-checked), identifiers and codes, dates,
region proper nouns, person names, and the layout itself (frames, styles, templates).

## Known limits

- The BRS **direct-table** rendering branch stays single-language; bilingual pairing
  applies to the Orchestrator Generate path for bilingual profiles with a non-empty memory.
- Manifest-based compilation and `pgs-cli compile` never seed translations; compiled output
  is single-language.
- Header/footer lines are not collected yet.
- Private-CA HTTPS endpoints are unsupported (see TLS note above).
- The direction is Indonesian → English only.
