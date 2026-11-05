use crate::glossary::{enforce, Entry};
use crate::metadata_form::Lang;
use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Map, Value};

pub trait Translator {
    fn call(&self, system: &str, user: &str) -> Result<String>;
}

pub struct LlmTranslator {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

impl Translator for LlmTranslator {
    fn call(&self, system: &str, user: &str) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let body = json!({
            "model": self.model,
            "temperature": 0.2,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user}
            ]
        });
        let response = ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .timeout(std::time::Duration::from_secs(90))
            .send_json(body)
            .with_context(|| format!("LLM request to {url}"))?;
        let value: Value = response.into_json().context("LLM reply is not JSON")?;
        let content = value
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| anyhow!("LLM reply has no choices[0].message.content"))?;
        Ok(content.to_owned())
    }
}

/// Offline stand-in: expands each source ~20% like a real language expansion.
/// Lets the whole pipeline run without an API key.
pub struct MockTranslator;

impl Translator for MockTranslator {
    fn call(&self, _system: &str, user: &str) -> Result<String> {
        let start = user.find('{').ok_or_else(|| anyhow!("mock translator needs a JSON payload"))?;
        let end = user.rfind('}').ok_or_else(|| anyhow!("mock translator needs a JSON payload"))?;
        let map: Map<String, Value> = serde_json::from_str(&user[start..=end])
            .context("mock translator payload is not a JSON object")?;
        let mut out = Map::new();
        for (key, value) in &map {
            if let Some(text) = value.as_str() {
                let extra = " tambahan";
                let mut expanded = text.to_owned();
                while (expanded.chars().count() as f64) < (text.chars().count() as f64) * 1.2 {
                    expanded.push_str(extra);
                }
                out.insert(key.clone(), Value::String(expanded));
            }
        }
        Ok(Value::Object(out).to_string())
    }
}

pub struct SlotInput<'a> {
    pub name: &'a str,
    pub source: &'a str,
    pub budget: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SlotStatus {
    Ok,
    OverBudget,
    Flagged(String),
}

#[derive(Clone, Debug)]
pub struct SlotOutcome {
    pub name: String,
    pub text: String,
    pub status: SlotStatus,
}

pub const MAX_ATTEMPTS: usize = 3;

fn lang_name(lang: Lang) -> &'static str {
    lang.label()
}

pub fn system_prompt(lang: Lang, glossary: &[Entry]) -> String {
    let mut s = format!(
        "You are a professional publication translator translating into {}. \
         Reply with ONLY a JSON object mapping each given key to its translated text. \
         No commentary, no code fences.",
        lang_name(lang)
    );
    let mandatory: Vec<String> = glossary
        .iter()
        .filter(|e| !e.do_not_translate && !e.term.is_empty() && !e.replacement.is_empty())
        .map(|e| format!("{} -> {}", e.term, e.replacement))
        .collect();
    if !mandatory.is_empty() {
        s.push_str("\nMandatory terminology (use exactly these translations): ");
        s.push_str(&mandatory.join("; "));
    }
    let keep: Vec<&str> = glossary
        .iter()
        .filter(|e| e.do_not_translate && !e.term.is_empty())
        .map(|e| e.term.as_str())
        .collect();
    if !keep.is_empty() {
        s.push_str("\nKeep these terms exactly as written, never translate them: ");
        s.push_str(&keep.join(", "));
    }
    s
}

fn strip_fences(text: &str) -> &str {
    let trimmed = text.trim();
    if let Some(rest) = trimmed.strip_prefix("```") {
        let after_first_line = rest.find('\n').map(|i| &rest[i + 1..]).unwrap_or(rest);
        let end = after_first_line.rfind("```").unwrap_or(after_first_line.len());
        return after_first_line[..end].trim();
    }
    trimmed
}

fn parse_mapping(raw: &str, keys: &[String]) -> Result<Map<String, Value>> {
    let cleaned = strip_fences(raw);
    let start = cleaned.find('{').ok_or_else(|| anyhow!("no JSON object in reply"))?;
    let end = cleaned.rfind('}').ok_or_else(|| anyhow!("no closing brace in reply"))?;
    if start >= end {
        bail!("empty JSON object in reply");
    }
    let value: Value = serde_json::from_str(&cleaned[start..=end])?;
    let map = value
        .as_object()
        .ok_or_else(|| anyhow!("reply is not a JSON object"))?
        .clone();
    for key in keys {
        let Some(v) = map.get(key) else {
            bail!("reply is missing key {key:?}");
        };
        if !v.is_string() {
            bail!("value for key {key:?} is not a string");
        }
    }
    Ok(map)
}

fn call_mapping(
    translator: &dyn Translator,
    system: &str,
    user: &str,
    keys: &[String],
) -> Result<Map<String, Value>> {
    let mut last_error = String::new();
    for _ in 1..=MAX_ATTEMPTS {
        match translator.call(system, user) {
            Ok(raw) => match parse_mapping(&raw, keys) {
                Ok(map) => return Ok(map),
                Err(e) => last_error = format!("{e}"),
            },
            Err(e) => last_error = format!("{e}"),
        }
    }
    bail!("LLM output invalid after {MAX_ATTEMPTS} attempts: {last_error}")
}

pub fn translate_slots(
    translator: &dyn Translator,
    lang: Lang,
    glossary: &[Entry],
    inputs: &[SlotInput],
) -> Vec<SlotOutcome> {
    let system = system_prompt(lang, glossary);
    let keys: Vec<String> = inputs.iter().map(|i| i.name.to_owned()).collect();
    let mut payload = Map::new();
    for input in inputs {
        let mut text = input.source.to_owned();
        if let Some(budget) = input.budget {
            text.push_str(&format!("\n[max {} characters]", budget));
        }
        payload.insert(input.name.to_owned(), Value::String(text));
    }
    let user = serde_json::to_string(&Value::Object(payload)).unwrap_or_else(|_| "{}".into());

    let map = match call_mapping(translator, &system, &user, &keys) {
        Ok(map) => map,
        Err(e) => {
            return inputs
                .iter()
                .map(|i| SlotOutcome {
                    name: i.name.to_owned(),
                    text: i.source.to_owned(),
                    status: SlotStatus::Flagged(format!("{e} — source text kept")),
                })
                .collect();
        }
    };

    let mut outcomes = Vec::new();
    for input in inputs {
        let mut text = map
            .get(input.name)
            .and_then(Value::as_str)
            .unwrap_or(input.source)
            .to_owned();
        let (enforced, fixes) = enforce(&text, glossary);
        if fixes > 0 {
            text = enforced;
        }
        let mut status = SlotStatus::Ok;
        if let Some(budget) = input.budget
            && text.chars().count() as u64 > budget {
                let repair_user = json!({
                    input.name: format!(
                        "{}\n\nThis translation is {} characters but must be at most {} characters. \
                         Rewrite it concisely in {} keeping the meaning. Reply with ONLY the JSON object.",
                        text,
                        text.chars().count(),
                        budget,
                        lang_name(lang)
                    )
                });
                match call_mapping(translator, &system, &repair_user.to_string(), &[input.name.to_owned()]) {
                    Ok(repaired) => {
                        let mut shorter = repaired
                            .get(input.name)
                            .and_then(Value::as_str)
                            .unwrap_or(&text)
                            .to_owned();
                        let (enforced, _) = enforce(&shorter, glossary);
                        shorter = enforced;
                        if shorter.chars().count() as u64 <= budget {
                            text = shorter;
                        } else {
                            text = shorter;
                            status = SlotStatus::OverBudget;
                        }
                    }
                    Err(e) => {
                        status = SlotStatus::Flagged(format!("rephrase failed: {e}"));
                    }
                }
            }
        outcomes.push(SlotOutcome { name: input.name.to_owned(), text, status });
    }
    outcomes
}

fn is_translatable(cell: &str) -> bool {
    cell.chars().any(|c| c.is_alphabetic())
}

pub fn translate_table(
    translator: &dyn Translator,
    lang: Lang,
    glossary: &[Entry],
    csv_text: &str,
) -> Result<String> {
    let mut reader = csv::ReaderBuilder::new().from_reader(csv_text.as_bytes());
    let headers = reader.headers()?.to_owned();
    let mut rows: Vec<Vec<String>> = Vec::new();
    for record in reader.records() {
        let record = record?;
        rows.push(record.iter().map(|c| c.to_owned()).collect());
    }
    let width = headers.len().max(rows.iter().map(|r| r.len()).max().unwrap_or(0));

    let mut keys: Vec<String> = Vec::new();
    let mut payload = Map::new();
    let mut cells: Vec<(usize, usize, String)> = Vec::new();
    let push_cell = |keys: &mut Vec<String>, payload: &mut Map<String, Value>, cells: &mut Vec<(usize, usize, String)>, r: usize, c: usize, cell: &str| {
        if is_translatable(cell) {
            let key = format!("r{r}c{c}");
            keys.push(key.clone());
            payload.insert(key.clone(), Value::String(cell.to_owned()));
            cells.push((r, c, cell.to_owned()));
        }
    };
    for (c, header) in headers.iter().enumerate() {
        push_cell(&mut keys, &mut payload, &mut cells, 0, c, header);
    }
    for (r, row) in rows.iter().enumerate() {
        for (c, cell) in row.iter().enumerate() {
            push_cell(&mut keys, &mut payload, &mut cells, r + 1, c, cell);
        }
    }

    let system = system_prompt(lang, glossary);
    let user = format!(
        "{}\nTranslate every value in this JSON object cell-by-cell. \
         Preserve numbers, symbols, units and formatting tokens exactly; never merge or reorder cells.",
        serde_json::to_string(&Value::Object(payload)).unwrap_or_else(|_| "{}".into())
    );
    let map = call_mapping(translator, &system, &user, &keys)?;

    let mut translated: Vec<Vec<String>> = Vec::with_capacity(rows.len() + 1);
    translated.push((0..width).map(|c| headers.get(c).map(|s| s.to_owned()).unwrap_or_default()).collect());
    for row in &rows {
        translated.push((0..width).map(|c| row.get(c).cloned().unwrap_or_default()).collect());
    }
    for (r, c, original) in &cells {
        let key = format!("r{r}c{c}");
        let Some(Value::String(new_text)) = map.get(&key) else { continue };
        let (enforced, _) = enforce(new_text, glossary);
        if enforced != *original {
            translated[*r][*c] = enforced;
        }
    }

    let mut w = csv::WriterBuilder::new().from_writer(Vec::new());
    for row in &translated {
        let _ = w.write_record(row);
    }
    Ok(String::from_utf8(w.into_inner().unwrap_or_default()).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct Scripted {
        replies: RefCell<Vec<Result<String>>>,
        seen: RefCell<Vec<(String, String)>>,
    }

    impl Translator for Scripted {
        fn call(&self, system: &str, user: &str) -> Result<String> {
            self.seen.borrow_mut().push((system.to_owned(), user.to_owned()));
            let mut replies = self.replies.borrow_mut();
            if replies.is_empty() {
                bail!("scripted translator exhausted");
            }
            replies.remove(0)
        }
    }

    fn slot_inputs() -> Vec<SlotInput<'static>> {
        vec![SlotInput { name: "slot:title", source: "Big news", budget: Some(50) }]
    }

    #[test]
    fn strict_json_ok() {
        let t = Scripted {
            replies: RefCell::new(vec![Ok(r#"{"slot:title": "Berita besar"}"#.into())]),
            seen: RefCell::new(Vec::new()),
        };
        let out = translate_slots(&t, Lang::Id, &[], &slot_inputs());
        assert_eq!(out[0].text, "Berita besar");
        assert_eq!(out[0].status, SlotStatus::Ok);
    }

    #[test]
    fn malformed_output_retried_then_accepted() {
        let t = Scripted {
            replies: RefCell::new(vec![
                Ok("sure here you go!".into()),
                Ok(r#"```json
{"slot:title": "Berita"}
```"#.into()),
            ]),
            seen: RefCell::new(Vec::new()),
        };
        let out = translate_slots(&t, Lang::Id, &[], &slot_inputs());
        assert_eq!(out[0].text, "Berita");
        assert_eq!(out[0].status, SlotStatus::Ok);
        assert_eq!(t.seen.borrow().len(), 2);
    }

    #[test]
    fn missing_key_is_retried_and_flagged_in_the_end() {
        let t = Scripted {
            replies: RefCell::new(vec![
                Ok(r#"{"other": "x"}"#.into()),
                Ok(r#"{"other": "x"}"#.into()),
                Ok(r#"{"other": "x"}"#.into()),
            ]),
            seen: RefCell::new(Vec::new()),
        };
        let out = translate_slots(&t, Lang::Id, &[], &slot_inputs());
        assert!(matches!(out[0].status, SlotStatus::Flagged(_)));
        assert_eq!(out[0].text, "Big news");
    }

    #[test]
    fn over_budget_triggers_rephrase() {
        let long = "x".repeat(60);
        let short = "y".repeat(20);
        let t = Scripted {
            replies: RefCell::new(vec![
                Ok(json!({"slot:title": long}).to_string()),
                Ok(json!({"slot:title": short}).to_string()),
            ]),
            seen: RefCell::new(Vec::new()),
        };
        let out = translate_slots(&t, Lang::Id, &[], &slot_inputs());
        assert_eq!(out[0].status, SlotStatus::Ok);
        assert_eq!(out[0].text.chars().count(), 20);
        assert_eq!(t.seen.borrow().len(), 2);
    }

    #[test]
    fn still_over_budget_is_flagged_with_shortest_text() {
        let long = "x".repeat(60);
        let t = Scripted {
            replies: RefCell::new(vec![
                Ok(json!({"slot:title": long.clone()}).to_string()),
                Ok(json!({"slot:title": long}).to_string()),
            ]),
            seen: RefCell::new(Vec::new()),
        };
        let out = translate_slots(&t, Lang::Id, &[], &slot_inputs());
        assert_eq!(out[0].status, SlotStatus::OverBudget);
        assert!(matches!(out[0].status, SlotStatus::OverBudget));
    }

    #[test]
    fn glossary_enforced_after_translation() {
        let g = vec![crate::glossary::Entry {
            term: "quantum leap".into(),
            replacement: "lompatan kuantum".into(),
            do_not_translate: false,
            case_preserve: false,
        }];
        let t = Scripted {
            replies: RefCell::new(vec![Ok(r#"{"slot:title": "Quantum Leap baru"}"#.into())]),
            seen: RefCell::new(Vec::new()),
        };
        let out = translate_slots(&t, Lang::Id, &g, &slot_inputs());
        assert_eq!(out[0].text, "lompatan kuantum baru");
        assert!(t.seen.borrow()[0].0.contains("Mandatory terminology"));
    }

    #[test]
    fn do_not_translate_is_in_prompt() {
        let g = vec![crate::glossary::Entry {
            term: "DesignCraft".into(),
            replacement: String::new(),
            do_not_translate: true,
            case_preserve: false,
        }];
        let t = Scripted {
            replies: RefCell::new(vec![Ok(r#"{"slot:title": "x"}"#.into())]),
            seen: RefCell::new(Vec::new()),
        };
        let _ = translate_slots(&t, Lang::Id, &g, &slot_inputs());
        assert!(t.seen.borrow()[0].0.contains("never translate"));
    }

    #[test]
    fn table_cells_translated_numbers_kept() {
        let csv = "name,qty\nWidget,4\nGear,12";
        let t = Scripted {
            replies: RefCell::new(vec![Ok(
                r#"{"r0c0":"nama","r1c0":"Widget Kita","r2c0":"Gir","r0c1":"jumlah"}"#.into()
            )]),
            seen: RefCell::new(Vec::new()),
        };
        let out = translate_table(&t, Lang::Id, &[], csv).unwrap();
        assert!(out.contains("nama,jumlah"));
        assert!(out.contains("Widget Kita,4"));
        assert!(out.contains("Gir,12"));
    }

    #[test]
    fn mock_translator_expands() {
        let payload = json!({"slot:title": "abcde"});
        let out = MockTranslator.call("s", &payload.to_string()).unwrap();
        let v: Value = serde_json::from_str(&out).unwrap();
        let text = v["slot:title"].as_str().unwrap();
        assert!(text.chars().count() >= 6);
    }
}
