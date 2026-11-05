// Shared canvas geometry cache used by the preview and content-tab thumbnails.

use std::hash::{Hash, Hasher};

use iced::widget::canvas;

/// Per-canvas geometry cache keyed by a content hash.
///
/// `canvas::Program::draw` only receives `&State`, so the key is stored with
/// interior mutability. Geometry is reused across frames (scrolling, hovering,
/// idle ticks) and only rebuilt when the content key or canvas bounds change.
#[derive(Default)]
pub(super) struct CanvasCache {
    key: std::cell::Cell<Option<u64>>,
    geometry: canvas::Cache,
}

impl CanvasCache {
    pub(super) fn draw(
        &self,
        renderer: &iced::Renderer,
        bounds: iced::Rectangle,
        key: u64,
        build: impl FnOnce(&mut canvas::Frame),
    ) -> Vec<canvas::Geometry> {
        if self.key.replace(Some(key)) != Some(key) {
            self.geometry.clear();
        }
        vec![self.geometry.draw(renderer, bounds.size(), build)]
    }
}

pub(super) fn content_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}
