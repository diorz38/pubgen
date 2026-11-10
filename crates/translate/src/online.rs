//! Groq (OpenAI-compatible) chat-completions client: batched JSON-array translation
//! with fence stripping, count-mismatch errors and 429 Retry-After backoff.
//!
//! The API key lives only in the Authorization header: never logged, never in an
//! error string, never written into a document. Without the \`groq\` feature the type
//! exists and every call returns \`TranslateError::Unavailable\`.

#[cfg(feature = "online")]
use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicBool, AtomicU32};

use super::{Lang, TranslateError};

/// Last `x-ratelimit-remaining-requests` value Groq returned for this key
/// (server-computed, so it already covers every consumer of the key).
/// `u32::MAX` = nothing seen yet.
static GROQ_REMAINING_REQUESTS: AtomicU32 = AtomicU32::new(u32::MAX);

/// Approximate remaining chat-completion requests on the configured Groq key
/// today, straight from Groq's own rate-limit response header. `None` until the
/// first request against the key in this app session.
pub fn remaining_requests() -> Option<u32> {
    match GROQ_REMAINING_REQUESTS.load(std::sync::atomic::Ordering::Relaxed) {
        u32::MAX => None,
        v => Some(v),
    }
}

/// Remember the rate-limit header from a chat-completions response (200 or 429).
/// Only Groq's own responses feed the Groq counter: a local server must not
/// clobber the hosted key's number.
#[cfg(feature = "online")]
fn store_remaining(r: &ureq::Response, base_url: &str) {
    if !base_url.starts_with("https://api.groq.com") {
        return;
    }
    if let Some(v) = r.header("x-ratelimit-remaining-requests").and_then(|v| v.trim().parse::<u32>().ok()) {
        GROQ_REMAINING_REQUESTS.store(v, Ordering::Relaxed);
    }
}

/// Units per request: small enough for free-tier rate caps, big enough to amortise. Raised for the
/// larger-context endpoints (custom OpenAI-compatible, local); the caller's per-request character
/// budget is what keeps one request from being truncated, not this count.
#[cfg(feature = "online")]
const BATCH: usize = 32;
#[cfg(feature = "online")]
const MAX_429_RETRIES: usize = 3;
/// Free tiers cap at 15 requests per minute. 4.4 s between requests keeps a run at ~13.6/min so a
/// burst never trips the limit; with the batched requests a whole publication is ~6 requests, so
/// the pacing costs seconds, not minutes.
#[cfg(feature = "online")]
const MIN_REQUEST_INTERVAL: std::time::Duration = std::time::Duration::from_millis(4_400);

/// Terminology pairs appended to the system prompt (bounded, sorted upstream).
#[cfg(feature = "online")]
const TERMS_IN_PROMPT: usize = 200;

/// Shared by both request shapes: the figures are the part a translator must not touch.
#[cfg(feature = "online")]
const FIGURE_RULES: &str = "Figures are data, never prose. Never translate, round, spell out or \"improve\" one:\n- Keep every number exactly as written: 27, 1.234, 1,93, 1.234,56, 12,5%, 68.4, 2026.\n- Keep ratios, comparisons and scores exactly as written: 30:1, 2:3, 10:7, 1/4, 3 of 10. Never rewrite 30:1 as \"30 to 1\", \"30 - 1\", \"3:10\" or words.\n- Keep ranges, dates, years, table numbers, codes and units exactly as written.\n- Keep a figure in the same sentence it belongs to, and keep the % sign on it.\n- Do not add a figure the source does not contain, and do not drop one it does.";

/// A chat model likes to wrap its answer in chatter — a preface, a note, an offer of options, a
/// question back. That is prose the cell must not keep: a table cell came back holding
/// "Next: what do you need data, translation, or stats?".
#[cfg(feature = "online")]
const ANSWER_ONLY_RULES: &str = "Answer with the translation only, as short as the source: no preface, no note, no explanation, no gloss in brackets, no options, and never a question back to the reader.";

/// Online translator (Groq free tier; OpenAI-compatible chat completions).
#[cfg_attr(not(feature = "online"), allow(dead_code))]
pub struct OnlineTranslator {
    key: String,
    model: String,
    #[cfg(feature = "online")]
    agent: ureq::Agent,
    /// Overridable for tests (a canned local listener stands in for api.groq.com).
    #[cfg(feature = "online")]
    base_url: String,
    /// When the previous request went out, so the next one can wait out the free tier's cap
    /// (15 requests per minute is the one the custom gateways enforce).
    #[cfg(feature = "online")]
    last_call: std::sync::Mutex<Option<std::time::Instant>>,
}

impl OnlineTranslator {
    /// Groq's hosted endpoint. Kept so existing callers are unchanged.
    pub fn new(key: &str, model: &str) -> Self {
        Self::with_endpoint(key, model, "https://api.groq.com/openai/v1")
    }

    /// Any OpenAI-compatible endpoint: Groq, a custom host, or a local server
    /// (Ollama, LM Studio, llama.cpp). An empty key means no Authorization header.
    pub fn with_endpoint(key: &str, model: &str, base_url: &str) -> Self {
        #[cfg(not(feature = "online"))]
        let _ = base_url;
        Self {
            key: key.trim().to_string(),
            model: model.trim().to_string(),
            #[cfg(feature = "online")]
            agent: ureq::AgentBuilder::new().timeout(std::time::Duration::from_secs(30)).build(),
            #[cfg(feature = "online")]
            base_url: base_url.trim().trim_end_matches('/').to_string(),
            #[cfg(feature = "online")]
            last_call: std::sync::Mutex::new(None),
        }
    }

    #[cfg(all(test, feature = "online"))]
    fn with_base(key: &str, model: &str, base_url: String) -> Self {
        Self::with_endpoint(key, model, &base_url)
    }

    /// The Authorization header value, or `None` when no key is configured.
    #[cfg(feature = "online")]
    fn auth_header(&self) -> Option<String> {
        // Local servers run without a key; "Bearer " with an empty token is rejected
        // by some of them and is never useful.
        (!self.key.is_empty()).then(|| format!("Bearer {}", self.key))
    }

    /// The model ids the account may use, sorted (Groq GET /openai/v1/models).
    pub fn list_models(&self) -> Result<Vec<String>, TranslateError> {
        #[cfg(feature = "online")]
        {
            let mut req = self.agent.get(&format!("{}/models", self.base_url));
            if let Some(auth) = self.auth_header() {
                req = req.set("Authorization", &auth);
            }
            let res = req.call();
            let body = read_body(res)?;
            let v: serde_json::Value = serde_json::from_str(&body).map_err(|e| TranslateError::Provider(format!("model list: {e}")))?;
            let mut ids: Vec<String> = v
                .get("data")
                .and_then(|d| d.as_array())
                .map(|a| a.iter().filter_map(|m| m.get("id").and_then(|i| i.as_str()).map(str::to_string)).collect())
                .unwrap_or_default();
            ids.sort();
            Ok(ids)
        }
        #[cfg(not(feature = "online"))]
        Err(TranslateError::Unavailable)
    }

    pub fn translate(
        &mut self,
        texts: &[String],
        src: Lang,
        tgt: Lang,
        glossary: Option<&[(String, String)]>,
        cancel: &AtomicBool,
        progress: &mut dyn FnMut(usize, usize),
    ) -> Result<Vec<String>, TranslateError> {
        #[cfg(feature = "online")]
        {
            let mut out = Vec::with_capacity(texts.len());
            for batch in texts.chunks(BATCH) {
                if cancel.load(Ordering::Relaxed) {
                    return Err(TranslateError::Cancelled);
                }
                out.extend(self.translate_batch(batch, src, tgt, glossary, cancel)?);
                progress(out.len(), texts.len());
            }
            Ok(out)
        }
        #[cfg(not(feature = "online"))]
        {
            let _ = (texts, src, tgt, glossary, cancel, progress);
            Err(TranslateError::Unavailable)
        }
    }

    #[cfg(feature = "online")]
    fn translate_batch(
        &self,
        batch: &[String],
        src: Lang,
        tgt: Lang,
        glossary: Option<&[(String, String)]>,
        cancel: &AtomicBool,
    ) -> Result<Vec<String>, TranslateError> {
        let mut attempt = 0usize;
        // Plan salvage path: when the batch output is unusable (non-JSON, wrong count —
        // some reasoning models prose-ify their answer), retry the batch item-by-item.
        let mut item_fallback = false;
        loop {
            if cancel.load(Ordering::Relaxed) {
                return Err(TranslateError::Cancelled);
            }
            let res = if item_fallback {
                let mut out = Vec::with_capacity(batch.len());
                for item in batch {
                    out.extend(self.request_once(std::slice::from_ref(item), src, tgt, glossary)?);
                }
                Ok(out)
            } else {
                self.request_once(batch, src, tgt, glossary)
            };
            match res {
                Ok(out) => return Ok(out),
                Err(TranslateError::RateLimited(retry_after)) => {
                    attempt += 1;
                    if attempt > MAX_429_RETRIES {
                        return Err(TranslateError::Provider(format!("rate limited after {MAX_429_RETRIES} retries")));
                    }
                    if cancel.load(Ordering::Relaxed) {
                        return Err(TranslateError::Cancelled);
                    }
                    std::thread::sleep(std::time::Duration::from_secs(retry_after));
                }
                Err(TranslateError::Provider(_)) if !item_fallback && batch.len() > 1 => {
                    item_fallback = true;
                    attempt = 0;
                }
                Err(e) => return Err(e),
            }
        }
    }

    #[cfg(feature = "online")]
    fn request_once(&self, batch: &[String], src: Lang, tgt: Lang, glossary: Option<&[(String, String)]>) -> Result<Vec<String>, TranslateError> {
        // One text does not need the array protocol, and asking for it is what broke against
        // gateways whose model splits a single item into many (it answered 49 items for a batch
        // of 1). A prose request has no count to get wrong.
        if let [only] = batch {
            return self.request_prose(only, src, tgt, glossary).map(|s| vec![s]);
        }
        let body = request_body(&self.model, batch, src, tgt, glossary);
        let raw = self.post_chat(&body)?;
        let v = json_body(&raw)?;
        let content = v
            .pointer("/choices/0/message/content")
            .and_then(|c| c.as_str())
            .ok_or_else(|| TranslateError::Provider("response has no choices[0].message.content".into()))?;
        let parsed = parse_translation(content)?;
        if parsed.len() != batch.len() {
            // Never silently reorder: name the batch and fail.
            return Err(TranslateError::Provider(format!(
                "model returned {} items for a batch of {} starting {:?}",
                parsed.len(),
                batch.len(),
                batch.first().map(|s| truncate(s, 40)).unwrap_or_default()
            )));
        }
        Ok(parsed.into_iter().map(|s| strip_chatter(&s)).collect())
    }

    /// One text, one prose answer: the salvage path when the array protocol is unusable (a model
    /// that answers with a different number of items, or with commentary around them). A fenced or
    /// array-shaped answer is still accepted — the caller only wants the words.
    #[cfg(feature = "online")]
    fn request_prose(&self, text: &str, src: Lang, tgt: Lang, glossary: Option<&[(String, String)]>) -> Result<String, TranslateError> {
        if text.trim().is_empty() {
            return Ok(String::new());
        }
        let body = prose_body(&self.model, text, src, tgt, glossary);
        let raw = self.post_chat(&body)?;
        let v = json_body(&raw)?;
        let content = v
            .pointer("/choices/0/message/content")
            .and_then(|c| c.as_str())
            .ok_or_else(|| TranslateError::Provider("response has no choices[0].message.content".into()))?
            .trim();
        let answer = if content.starts_with('[') || content.starts_with('`') {
            match parse_translation(content) {
                Ok(items) if !items.is_empty() => items.join(" "),
                _ => content.to_string(),
            }
        } else {
            content.trim_matches('"').trim().to_string()
        };
        let answer = strip_chatter(&answer);
        if answer.trim().is_empty() {
            return Err(TranslateError::Provider("the model returned an empty translation".into()));
        }
        Ok(answer)
    }

    /// One chat completion with a caller-supplied prompt (the narrative drafter). Same auth,
    /// status and `Retry-After` handling as the translation call; `temperature: 0`.
    pub fn complete(&self, system: &str, user: &str) -> Result<String, TranslateError> {
        #[cfg(feature = "online")]
        {
            let body = chat_body(&self.model, system, user);
            let mut attempt = 0usize;
            let raw = loop {
                match self.post_chat(&body) {
                    Err(TranslateError::RateLimited(retry_after)) if attempt < MAX_429_RETRIES => {
                        attempt += 1;
                        std::thread::sleep(std::time::Duration::from_secs(retry_after));
                    }
                    other => break other?,
                }
            };
            let v = json_body(&raw)?;
            v.pointer("/choices/0/message/content")
                .and_then(|c| c.as_str())
                .map(str::to_string)
                .ok_or_else(|| TranslateError::Provider("response has no choices[0].message.content".into()))
        }
        #[cfg(not(feature = "online"))]
        {
            let _ = (system, user);
            Err(TranslateError::Unavailable)
        }
    }

    /// Wait out the free tier's request spacing before sending. Spacing is cheaper than being
    /// rate-limited: a 429 costs a retry plus the gateway's own advertised wait.
    #[cfg(feature = "online")]
    fn pace(&self) {
        if cfg!(test) {
            return; // the suite's canned listener must not pay the pacing delay
        }
        let mut last = self.last_call.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(prev) = *last {
            let gap = MIN_REQUEST_INTERVAL.saturating_sub(prev.elapsed());
            if !gap.is_zero() {
                std::thread::sleep(gap);
            }
        }
        *last = Some(std::time::Instant::now());
    }

    /// POST one chat-completions body; the model's raw JSON reply on 200.
    #[cfg(feature = "online")]
    fn post_chat(&self, body: &str) -> Result<String, TranslateError> {
        self.pace();
        let mut req = self.agent.post(&format!("{}/chat/completions", self.base_url)).set("Content-Type", "application/json");
        if let Some(auth) = self.auth_header() {
            req = req.set("Authorization", &auth);
        }
        let res = req.send_string(body);
        let (status, raw) = match res {
            Ok(r) => {
                store_remaining(&r, &self.base_url);
                (r.status(), r.into_string().map_err(|e| TranslateError::Provider(e.to_string()))?)
            }
            Err(ureq::Error::Status(code, r)) => {
                store_remaining(&r, &self.base_url);
                let retry = r.header("Retry-After").and_then(|v| v.trim().parse::<u64>().ok()).map(|s| s.clamp(1, 30));
                let raw = r.into_string().map_err(|e| TranslateError::Provider(e.to_string()))?;
                if code == 429 {
                    return Err(TranslateError::RateLimited(retry.unwrap_or(2)));
                }
                // Some gateways wrap their rate limit in a 503 and put the real 429 in the body.
                // Treating that as a plain provider error ended a 178-text run after 64 of them.
                if let Some(wait) = rate_limit_wait(&raw) {
                    return Err(TranslateError::RateLimited(retry.unwrap_or(wait)));
                }
                // Never echo the key: the body names the endpoint problem only.
                (code, raw)
            }
            Err(e) => return Err(TranslateError::Provider(format!("request failed: {e}"))),
        };
        if status != 200 {
            return Err(TranslateError::Provider(format!("HTTP {status}: {}", truncate(&raw, 300))));
        }
        Ok(raw)
    }
}

/// The wait a rate-limited answer asks for, when a gateway reports its 429 inside another status
/// (`HTTP 503 ... "[429]: Rate limit exceeded. Max 15 requests per minute ... Retry in 24s."`).
/// `None` when the body says nothing about a rate limit.
#[cfg(feature = "online")]
fn rate_limit_wait(raw: &str) -> Option<u64> {
    let lower = raw.to_ascii_lowercase();
    if !(lower.contains("429") || lower.contains("rate limit") || lower.contains("too many requests")) {
        return None;
    }
    let asked = lower.find("retry").and_then(|at| {
        lower[at..].chars().skip_while(|c| !c.is_ascii_digit()).take_while(char::is_ascii_digit).collect::<String>().parse::<u64>().ok()
    });
    Some(asked.unwrap_or(2).clamp(1, 60))
}

/// Trailing lines a chat model wraps around its answer: a preface label, a note, an offer of
/// options, a question back. Only the END of the answer is trimmed — a "Note:" that is part of the
/// source text stays.
#[cfg(feature = "online")]
fn strip_chatter(text: &str) -> String {
    let mut lines: Vec<&str> = text.lines().collect();
    while let Some(last) = lines.last() {
        let line = last.trim();
        if line.is_empty() || is_chatter_line(line) {
            lines.pop();
        } else {
            break;
        }
    }
    if lines.is_empty() { text.trim().to_string() } else { lines.join("\n") }
}

#[cfg(feature = "online")]
fn is_chatter_line(line: &str) -> bool {
    let low = line.to_ascii_lowercase();
    const PREFIXES: [&str; 10] =
        ["next:", "next :", "note:", "note :", "translation:", "here is", "here's", "would you", "do you want", "let me know"];
    if PREFIXES.iter().any(|p| low.starts_with(p)) {
        return true;
    }
    // A question offering the reader choices instead of a translation.
    low.contains('?') && ["do you", "would you", "shall i", "want me", "need me", "let me know", "what do you need"].iter().any(|p| low.contains(p))
}

/// One `temperature: 0` chat completion with a caller-supplied system/user prompt.
#[cfg(feature = "online")]
fn chat_body(model: &str, system: &str, user: &str) -> String {
    let mut body = serde_json::json!({
        "model": model,
        "temperature": 0,
        "stream": false,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user},
        ],
    });
    if model.contains("gpt-oss")
        && let Some(obj) = body.as_object_mut()
    {
        obj.insert("reasoning_effort".into(), serde_json::json!("low"));
    }
    body.to_string()
}

/// Read a chat-completion answer the way "OpenAI-compatible" gateways actually send it: as plain
/// JSON, as an SSE `data: {...}` stream (several ignore `stream: false`), or as one object with
/// something after it. A strict `from_str` reported those last two as
/// `bad JSON: trailing characters at line 1 column N`, which blamed the base URL for a server
/// quirk. Only the FIRST JSON value is kept.
#[cfg(feature = "online")]
fn json_body(raw: &str) -> Result<serde_json::Value, TranslateError> {
    let text = raw.trim_start_matches('\u{feff}');
    if let Some(v) = sse_body(text) {
        return Ok(v);
    }
    match serde_json::Deserializer::from_str(text).into_iter::<serde_json::Value>().next() {
        Some(Ok(v)) => Ok(v),
        Some(Err(e)) => Err(TranslateError::Provider(format!("bad JSON: {e} — the endpoint answered: {}", truncate(text.trim(), 160)))),
        None => Err(TranslateError::Provider("the endpoint returned an empty body".into())),
    }
}

/// The whole answer rebuilt from an SSE stream, if `text` is one: a streamed chunk carries its
/// words in `choices[0].delta.content`, a non-streamed object in `choices[0].message.content`.
#[cfg(feature = "online")]
fn sse_body(text: &str) -> Option<serde_json::Value> {
    if !text.trim_start().starts_with("data:") {
        return None;
    }
    let mut prose = String::new();
    let mut whole = None;
    for line in text.lines() {
        let Some(payload) = line.trim_start().strip_prefix("data:") else { continue };
        let payload = payload.trim();
        if payload.is_empty() || payload == "[DONE]" {
            continue;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(payload) else { continue };
        if v.pointer("/choices/0/message/content").is_some() {
            whole = Some(v);
        } else if let Some(t) = v.pointer("/choices/0/delta/content").and_then(|c| c.as_str()) {
            prose.push_str(t);
        }
    }
    whole.or_else(|| (!prose.is_empty()).then(|| serde_json::json!({"choices": [{"message": {"content": prose}}]})))
}

/// Units per request kept at module scope for the tests that build batches.
#[cfg(feature = "online")]
fn request_body(model: &str, batch: &[String], src: Lang, tgt: Lang, glossary: Option<&[(String, String)]>) -> String {
    let items = serde_json::to_string(batch).unwrap_or_default();
    let mut system = format!(
        "You are a translation engine. Translate each array item from {} to {}. Return a JSON \
         array of the same length and order. Keep numbers, proper nouns and formatting unchanged. \
         Output only the JSON array.\n\n{}\n\n{}",
        src.label(),
        tgt.label(),
        FIGURE_RULES,
        ANSWER_ONLY_RULES
    );
    push_glossary_terms(&mut system, glossary);
    // Reasoning models burn hidden reasoning tokens that count against the provider's
    // TPM and add nothing to a translation. Pin the effort to low when supported.
    let mut body = serde_json::json!({
        "model": model,
        "temperature": 0,
        "stream": false,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": items},
        ],
    });
    if model.contains("gpt-oss")
        && let Some(obj) = body.as_object_mut()
    {
        obj.insert("reasoning_effort".into(), serde_json::json!("low"));
    }
    body.to_string()
}

/// The one-text request of the salvage path: no array, no count to get wrong.
#[cfg(feature = "online")]
fn prose_body(model: &str, text: &str, src: Lang, tgt: Lang, glossary: Option<&[(String, String)]>) -> String {
    let mut system = format!(
        "You are a translation engine. Translate the text from {} to {}. Keep the paragraph \
         breaks. Return only the translation: no commentary, no quotes, no code fence.\n\n{}\n\n{}",
        src.label(),
        tgt.label(),
        FIGURE_RULES,
        ANSWER_ONLY_RULES
    );
    push_glossary_terms(&mut system, glossary);
    let mut body = serde_json::json!({
        "model": model,
        "temperature": 0,
        "stream": false,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": text},
        ],
    });
    if model.contains("gpt-oss")
        && let Some(obj) = body.as_object_mut()
    {
        obj.insert("reasoning_effort".into(), serde_json::json!("low"));
    }
    body.to_string()
}

/// Secondary net behind the deterministic whole-cell aliases: the terms shape the model's word
/// choice inside sentences. Sorted and capped so the prompt is stable and bounded; `None` and an
/// empty list leave the prompt unchanged.
#[cfg(feature = "online")]
fn push_glossary_terms(system: &mut String, glossary: Option<&[(String, String)]>) {
    if let Some(terms) = glossary.filter(|t| !t.is_empty()) {
        // An identity pair is a "don't translate" term: say so instead of listing `X = X` among the
        // terminology, which would read as a translation. Masking already keeps it out of the text.
        let is_keep = |(id, en): &(String, String)| id.eq_ignore_ascii_case(en);
        let translate: Vec<String> = terms.iter().filter(|p| !is_keep(p)).take(TERMS_IN_PROMPT).map(|(id, en)| format!("{id} = {en}")).collect();
        if !translate.is_empty() {
            system.push_str(&format!("\n\nOfficial terminology you MUST use: {}.", translate.join("; ")));
        }
        let kept: Vec<String> = terms.iter().filter(|p| is_keep(p)).take(TERMS_IN_PROMPT).map(|(id, _)| id.clone()).collect();
        if !kept.is_empty() {
            system.push_str(&format!("\n\nNever translate these terms, keep them exactly as written: {}.", kept.join("; ")));
        }
    }
}

/// Strip a triple-backtick json fence if present, then parse the JSON array.
#[cfg(feature = "online")]
fn parse_translation(content: &str) -> Result<Vec<String>, TranslateError> {
    let fence: String = "`".repeat(3);
    let mut s = content.trim();
    if let Some(rest) = s.strip_prefix(&fence) {
        // The opening fence line may name the language.
        let body = rest.split_once('\n').map(|(_, body)| body).unwrap_or(rest);
        s = body.trim_end();
        if let Some(without) = s.strip_suffix(&fence) {
            s = without.trim_end();
        }
    }
    let v: serde_json::Value = serde_json::from_str(s).map_err(|e| TranslateError::Provider(format!("model output is not JSON: {e}")))?;
    let arr = v.as_array().ok_or_else(|| TranslateError::Provider("model output is not a JSON array".into()))?;
    Ok(arr.iter().map(|i| i.as_str().unwrap_or_default().to_string()).collect())
}

#[cfg(feature = "online")]
fn read_body(res: Result<ureq::Response, ureq::Error>) -> Result<String, TranslateError> {
    match res {
        Ok(r) => r.into_string().map_err(|e| TranslateError::Provider(e.to_string())),
        Err(ureq::Error::Status(429, r)) => {
            let retry = r.header("Retry-After").and_then(|v| v.trim().parse::<u64>().ok()).map(|s| s.clamp(1, 30));
            Err(TranslateError::RateLimited(retry.unwrap_or(2)))
        }
        Err(ureq::Error::Status(code, r)) => {
            let raw = r.into_string().map_err(|e| TranslateError::Provider(e.to_string()))?;
            Err(TranslateError::Provider(format!("HTTP {code}: {}", truncate(&raw, 300))))
        }
        Err(e) => Err(TranslateError::Provider(format!("request failed: {e}"))),
    }
}

#[cfg(feature = "online")]
fn truncate(s: &str, n: usize) -> &str {
    match s.char_indices().nth(n) {
        Some((i, _)) => &s[..i],
        None => s,
    }
}

pub fn compiled_in() -> bool {
    cfg!(feature = "online")
}

#[cfg(all(test, feature = "online"))]
mod tests {
    use super::*;

    #[test]
    fn parse_strips_json_fence() {
        let fence: String = std::iter::repeat('\u{60}').take(3).collect();
        let fenced = format!("{fence}json\n[\"Halo\", \"Dunia\"]\n{fence}");
        let out = parse_translation(&fenced).expect("fenced array parses");
        assert_eq!(out, vec!["Halo", "Dunia"]);
        let out = parse_translation("[\"a\", \"b\"]").expect("bare array parses");
        assert_eq!(out, vec!["a", "b"]);
        let prose = "Sure! Here is the translation: [\"a\"]";
        let err = parse_translation(prose).err().expect("prose is an error");
        assert!(matches!(err, TranslateError::Provider(_)));
        let obj = "{\"a\": 1}";
        let err = parse_translation(obj).err().expect("object is an error");
        assert!(matches!(err, TranslateError::Provider(_)));
    }

    #[test]
    fn request_body_pins_the_contract() {
        let body = request_body("llama-3.3-70b", &["Kabupaten Karawang".into()], Lang::Indonesian, Lang::English, None);
        let v: serde_json::Value = serde_json::from_str(&body).expect("request body is valid JSON");
        assert_eq!(v.pointer("/model"), Some(&serde_json::json!("llama-3.3-70b")));
        assert_eq!(v.pointer("/temperature"), Some(&serde_json::json!(0)));
        let system = v.pointer("/messages/0/content").and_then(|c| c.as_str()).expect("system message");
        assert!(system.contains("Indonesian to English"), "{system}");
        assert!(system.contains("same length and order"), "{system}");
        // The user message is a JSON-encoded array (escaped inside the outer JSON).
        let user = v.pointer("/messages/1/content").and_then(|c| c.as_str()).expect("user message");
        assert_eq!(user, "[\"Kabupaten Karawang\"]");
    }

    #[test]
    fn empty_glossary_keeps_the_prompt_byte_identical() {
        let plain = request_body("m", &["Jumlah".into()], Lang::Indonesian, Lang::English, None);
        let empty = request_body("m", &["Jumlah".into()], Lang::Indonesian, Lang::English, Some(&[]));
        assert_eq!(plain, empty);
        assert!(!plain.contains("Official terminology"), "{plain}");
    }

    #[test]
    fn a_keep_term_is_asked_as_never_translate_not_as_a_translation() {
        let terms = vec![("Citeureup".to_string(), "Citeureup".to_string()), ("Kecamatan".to_string(), "Subdistrict".to_string())];
        let body = request_body("m", &["Kecamatan Citeureup".into()], Lang::Indonesian, Lang::English, Some(&terms));
        let v: serde_json::Value = serde_json::from_str(&body).expect("request body is valid JSON");
        let system = v.pointer("/messages/0/content").and_then(|c| c.as_str()).expect("system message");
        assert!(system.contains("Never translate these terms, keep them exactly as written: Citeureup."), "{system}");
        assert!(system.contains("Kecamatan = Subdistrict"), "{system}");
        assert!(!system.contains("Citeureup = Citeureup"), "an identity pair is not a translation: {system}");
    }

    #[test]
    fn glossary_terms_join_the_system_prompt() {
        let terms = vec![("Kabupaten".to_string(), "Regency".to_string()), ("Kecamatan".to_string(), "Subdistrict".to_string())];
        let body = request_body("m", &["Kecamatan Karawang".into()], Lang::Indonesian, Lang::English, Some(&terms));
        let v: serde_json::Value = serde_json::from_str(&body).expect("request body is valid JSON");
        let system = v.pointer("/messages/0/content").and_then(|c| c.as_str()).expect("system message");
        assert!(system.contains("Official terminology"), "{system}");
        assert!(system.contains("Kecamatan = Subdistrict"), "{system}");
        assert!(system.contains("Kabupaten = Regency"), "{system}");
        let user = v.pointer("/messages/1/content").and_then(|c| c.as_str()).expect("user message");
        assert_eq!(user, "[\"Kecamatan Karawang\"]", "the glossary must not leak into the payload");
    }

    #[test]
    fn glossary_terms_are_capped_at_the_prompt_budget() {
        let terms: Vec<(String, String)> = (0..TERMS_IN_PROMPT + 1).map(|i| (format!("istilah{i:03}"), format!("term{i:03}"))).collect();
        let body = request_body("m", &["x".into()], Lang::Indonesian, Lang::English, Some(&terms));
        let v: serde_json::Value = serde_json::from_str(&body).expect("request body is valid JSON");
        let system = v.pointer("/messages/0/content").and_then(|c| c.as_str()).expect("system message");
        assert!(system.contains("istilah199 = term199"), "{system}");
        assert!(!system.contains("istilah200 = term200"), "the 201st term must be dropped");
    }

    #[test]
    fn chat_body_pins_zero_temperature_and_both_prompts() {
        let body = chat_body("llama-3.3-70b", "system prompt", "user payload");
        let v: serde_json::Value = serde_json::from_str(&body).expect("chat body is valid JSON");
        assert_eq!(v.pointer("/temperature"), Some(&serde_json::json!(0)));
        assert_eq!(v.pointer("/messages/0/role"), Some(&serde_json::json!("system")));
        assert_eq!(v.pointer("/messages/0/content"), Some(&serde_json::json!("system prompt")));
        assert_eq!(v.pointer("/messages/1/role"), Some(&serde_json::json!("user")));
        assert_eq!(v.pointer("/messages/1/content"), Some(&serde_json::json!("user payload")));
        assert!(v.get("reasoning_effort").is_none(), "only gpt-oss gets the effort hint");
        let oss: serde_json::Value = serde_json::from_str(&chat_body("openai/gpt-oss-20b", "s", "u")).expect("valid JSON");
        assert_eq!(oss.get("reasoning_effort"), Some(&serde_json::json!("low")));
    }

    #[test]
    fn complete_round_trips_against_a_local_listener() {
        let content = "{\"blocks\":[{\"role\":\"lead\",\"text\":\"Hello\"}]}";
        let reply = serde_json::json!({"choices": [{"message": {"content": content}}]}).to_string();
        let base = canned_server(reply);
        let t = OnlineTranslator::with_base("test-key", "llama-3.3-70b", base);
        assert_eq!(t.complete("system", "user").expect("completion"), content);
    }

    /// A canned TCP listener stands in for api.groq.com: one POST, one JSON reply.
    fn canned_server(reply_body: String) -> String {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            use std::io::{Read, Write};
            // Read the whole request (headers + Content-Length body) before replying: a
            // partial read would close the socket mid-request and reset the client.
            let mut req: Vec<u8> = Vec::new();
            let mut buf = [0u8; 8192];
            loop {
                let n = match stream.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => n,
                };
                req.extend_from_slice(&buf[..n]);
                let header_end = req.windows(4).position(|w| w == b"\r\n\r\n").map(|p| p + 4);
                if let Some(body_start) = header_end {
                    let cl: usize = String::from_utf8_lossy(&req)
                        .split("\r\n")
                        .find_map(|l| l.to_ascii_lowercase().strip_prefix("content-length:").and_then(|v| v.trim().parse::<usize>().ok()))
                        .unwrap_or(0);
                    if req.len() >= body_start + cl {
                        break;
                    }
                }
            }
            let reply = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nx-ratelimit-remaining-requests: 812\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                reply_body.len(),
                reply_body
            );
            let _ = stream.write_all(reply.as_bytes());
            let _ = stream.flush();
        });
        format!("http://{addr}/v1")
    }

    fn content_array(items: &[&str]) -> String {
        let quoted: Vec<String> = items.iter().map(|i| format!("\"{i}\"")).collect();
        format!("[{}]", quoted.join(", ")).replace('"', "\\\"")
    }

    #[test]
    fn translate_round_trip_against_a_local_listener() {
        let base = canned_server(format!("{{\"choices\":[{{\"message\":{{\"content\":\"{}\"}}}}]}}", content_array(&["Karawang Timur Subdistrict"])));
        let mut t = OnlineTranslator::with_base("test-key", "llama-3.3-70b", base);
        let cancel = AtomicBool::new(false);
        let out =
            t.translate(&["Kecamatan Karawang Timur".into()], Lang::Indonesian, Lang::English, None, &cancel, &mut |_, _| {}).expect("round trip");
        assert_eq!(out, vec!["Karawang Timur Subdistrict"]);
        // The canned reply carries x-ratelimit-remaining-requests: 812, but the local
        // listener is not Groq: it must not clobber the hosted key's counter.
        assert_eq!(remaining_requests(), None);
    }

    #[test]
    fn translate_reports_a_count_mismatch_as_an_error() {
        let base = canned_server(format!("{{\"choices\":[{{\"message\":{{\"content\":\"{}\"}}}}]}}", content_array(&["satu"])));
        let mut t = OnlineTranslator::with_base("test-key", "m", base);
        let cancel = AtomicBool::new(false);
        let err = t
            .translate(&["satu".into(), "dua".into()], Lang::Indonesian, Lang::English, None, &cancel, &mut |_, _| {})
            .err()
            .expect("mismatch is an error");
        assert!(matches!(err, TranslateError::Provider(_)), "{err}");
    }

    #[test]
    fn with_endpoint_targets_the_supplied_base_url() {
        let base = canned_server("{\"choices\":[{\"message\":{\"content\":\"[\\\"Regency\\\"]\"}}]}".to_string());
        let mut t = OnlineTranslator::with_endpoint("", "local-model", &base);
        let cancel = AtomicBool::new(false);
        let out =
            t.translate(&["Kabupaten".into()], Lang::Indonesian, Lang::English, None, &cancel, &mut |_, _| {}).expect("local endpoint round trip");
        assert_eq!(out, vec!["Regency"]);
    }

    #[test]
    fn empty_key_sends_no_authorization_header() {
        // A local Ollama/LM Studio server has no key. The request must still succeed,
        // and the client must not send "Bearer " with an empty token.
        let base = canned_server("{\"choices\":[{\"message\":{\"content\":\"[\\\"Subdistrict\\\"]\"}}]}".to_string());
        let mut t = OnlineTranslator::with_endpoint("   ", "m", &base);
        let cancel = AtomicBool::new(false);
        assert!(t.translate(&["Kecamatan".into()], Lang::Indonesian, Lang::English, None, &cancel, &mut |_, _| {}).is_ok());
    }

    #[test]
    fn a_gateway_that_streams_anyway_is_read_as_whole_chunks() {
        // Several "OpenAI-compatible" gateways answer with `data: {...}` even though the body
        // asked for `stream: false`. That used to fail the setup check with
        // "bad JSON: trailing characters at line 1 column N" and blame the base URL.
        let chunk = |t: &str| format!("data: {}\n\n", serde_json::json!({"choices": [{"delta": {"content": t}}]}));
        let reply = format!("{}{}data: [DONE]\n\n", chunk("[\"Regency\""), chunk("]"));
        let base = canned_server(reply);
        let mut t = OnlineTranslator::with_endpoint("", "m", &base);
        let cancel = AtomicBool::new(false);
        let out = t
            .translate(&["Kabupaten".into()], Lang::Indonesian, Lang::English, None, &cancel, &mut |_, _| {})
            .expect("a streamed reply is still a reply");
        assert_eq!(out, vec!["Regency"]);
    }

    #[test]
    fn a_second_object_after_the_answer_is_ignored() {
        let reply = format!("{}", serde_json::json!({"choices": [{"message": {"content": "[\"Village\"]"}}]}));
        let reply = format!("{reply}{}", serde_json::json!({"note": "a gateway log line"}));
        let base = canned_server(reply);
        let mut t = OnlineTranslator::with_endpoint("", "m", &base);
        let cancel = AtomicBool::new(false);
        let out =
            t.translate(&["Desa".into()], Lang::Indonesian, Lang::English, None, &cancel, &mut |_, _| {}).expect("the first object is the answer");
        assert_eq!(out, vec!["Village"]);
    }

    #[test]
    fn requests_ask_for_a_non_streamed_answer() {
        let body = request_body("m", &["Desa".into()], Lang::Indonesian, Lang::English, None);
        let v: serde_json::Value = serde_json::from_str(&body).expect("body is JSON");
        assert_eq!(v.get("stream").and_then(|s| s.as_bool()), Some(false), "{body}");
        let plain = chat_body("m", "system", "user");
        let v: serde_json::Value = serde_json::from_str(&plain).expect("body is JSON");
        assert_eq!(v.get("stream").and_then(|s| s.as_bool()), Some(false), "{plain}");
    }

    #[test]
    fn a_429_hidden_inside_a_503_is_read_as_a_rate_limit() {
        // The gateway answered HTTP 503 with the real 429 in the body; that used to end a run.
        let body = r#"{"error":{"message":"[x/free/deepseek-v4.1-flash] [429]: {\"error\":{\"message\":\"Rate limit exceeded. Max 15 requests per minute for this account. Retry in 24s.\"}}"}}"#;
        assert_eq!(rate_limit_wait(body), Some(24));
        assert_eq!(rate_limit_wait("HTTP 503: upstream unavailable"), None);
        assert_eq!(rate_limit_wait("rate limited, retry soon"), Some(2));
    }

    #[test]
    fn requests_are_spaced_under_the_free_tier_limit() {
        // 15 requests per minute is the cap the custom gateways enforce; 4.4 s is ~13.6/min.
        let secs = MIN_REQUEST_INTERVAL.as_secs_f32();
        assert!((4.0..5.0).contains(&secs), "{secs}");
    }

    #[test]
    fn the_chatter_after_a_translation_is_cut_off() {
        let answered = "MA = Madrasah Aliyah\nLevel: SMA-equivalent (Grade 10-12)\nNext: what do you need data, translation, or stats?";
        let cut = strip_chatter(answered);
        assert!(!cut.contains("Next:"), "{cut}");
        assert!(cut.contains("MA = Madrasah Aliyah"), "{cut}");

        // A question back with no label goes too; a source sentence that asks something stays.
        assert_eq!(strip_chatter("Terjemahan\nWould you like a shorter version?"), "Terjemahan");
        assert_eq!(strip_chatter("Apakah data tersedia?"), "Apakah data tersedia?");

        // Both request shapes ask for the answer alone.
        let prose = prose_body("m", "teks", Lang::Indonesian, Lang::English, None);
        assert!(prose.contains("question back"), "{prose}");
        let array = request_body("m", &["teks".to_string(), "dua".to_string()], Lang::Indonesian, Lang::English, None);
        assert!(array.contains("question back"), "{array}");
    }

    #[test]
    fn a_single_text_is_asked_as_prose_not_as_an_array() {
        // Asking one text with the array protocol is what a gateway answered with 49 items.
        let body = prose_body("m", "Kecamatan Karawang Timur", Lang::Indonesian, Lang::English, None);
        assert!(body.contains("Kecamatan Karawang Timur"));
        assert!(!body.contains("same length and order"), "{body}");
        assert!(body.contains("30:1"), "the figure rules ride along: {body}");
    }
}
