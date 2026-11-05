use crate::control_client::ControlClient;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Slot {
    pub name: String,
    pub frame: u64,
    pub story: u64,
    #[serde(default)]
    pub budget: Option<u64>,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub text: String,
}

fn walk_items(value: &Value, found: &mut Vec<(u64, String)>) {
    match value {
        Value::Array(items) => items.iter().for_each(|v| walk_items(v, found)),
        Value::Object(map) => {
            let id = map.get("id").and_then(Value::as_u64);
            let name = map.get("name").and_then(Value::as_str);
            if let (Some(id), Some(name)) = (id, name)
                && name.starts_with("slot:") {
                    found.push((id, name.to_owned()));
                }
            if let Some(child) = map.get("children") {
                walk_items(child, found);
            }
        }
        _ => {}
    }
}

fn story_frames(value: &Value) -> Vec<(u64, Vec<u64>)> {
    let mut out = Vec::new();
    if let Some(stories) = value.get("stories").and_then(Value::as_array) {
        for s in stories {
            let id = s.get("id").and_then(Value::as_u64);
            let frames: Vec<u64> = s
                .get("frames")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_u64).collect())
                .unwrap_or_default();
            if let Some(id) = id {
                out.push((id, frames));
            }
        }
    }
    out
}

pub fn discover(client: &mut ControlClient) -> Result<Vec<Slot>> {
    let doc = client.inspect().context("document.inspect failed")?;
    let mut named = Vec::new();
    if let Some(spreads) = doc.get("spreads").and_then(Value::as_array) {
        for spread in spreads {
            if let Some(items) = spread.get("items") {
                walk_items(items, &mut named);
            }
        }
    }
    if let Some(items) = doc.get("items") {
        walk_items(items, &mut named);
    }
    let stories = story_frames(&doc);

    let mut slots = Vec::new();
    for (frame, name) in named {
        let story = stories
            .iter()
            .find(|(_, frames)| frames.contains(&frame))
            .map(|(id, _)| *id)
            .with_context(|| format!("slot {name}: frame {frame} has no story"))?;
        slots.push(Slot { name, frame, story, budget: None, source: String::new(), text: String::new() });
    }
    slots.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(slots)
}

pub fn fill_live(client: &mut ControlClient, slots: &[Slot]) -> Result<usize> {
    let mut filled = 0;
    for slot in slots {
        client.call("story.setText", json!({"story": slot.story, "text": slot.text}))?;
        filled += 1;
    }
    Ok(filled)
}

pub fn assign_name(client: &mut ControlClient, frame: u64, name: &str) -> Result<()> {
    client.call("object.rename", json!({"id": frame, "name": name}))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_inspect_document() {
        let doc: Value = serde_json::from_str(
            r#"{"spreads":[{"index":0,"items":[{"id":10,"name":"slot:title","kind":"Text"},{"id":11,"name":"body"}]}],
                "stories":[{"id":90,"frames":[10],"length":4},{"id":91,"frames":[11]}]}"#,
        )
        .unwrap();
        let mut named = Vec::new();
        if let Some(spreads) = doc.get("spreads").and_then(Value::as_array) {
            for spread in spreads {
                if let Some(items) = spread.get("items") {
                    walk_items(items, &mut named);
                }
            }
        }
        assert_eq!(named, vec![(10u64, "slot:title".to_owned())]);
        let stories = story_frames(&doc);
        let story = stories.iter().find(|(_, f)| f.contains(&10)).map(|(id, _)| *id);
        assert_eq!(story, Some(90));
    }

    #[test]
    fn finds_grouped_slots() {
        let mut named = Vec::new();
        let v: Value = serde_json::from_str(r#"[{"id":1,"name":"x","children":[{"id":2,"name":"slot:a"}]}]"#).unwrap();
        walk_items(&v, &mut named);
        assert_eq!(named, vec![(2u64, "slot:a".to_owned())]);
    }
}
