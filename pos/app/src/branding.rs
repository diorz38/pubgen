//! Shared branding artwork: the POS (Publication Orchestrator Studio) app icon.

/// 256 px PNG used for runtime window icons and in-app branding.
pub const ICON_PNG: &[u8] =
    include_bytes!("../../assets/app-icon/hicolor/256x256/apps/ai.storyteller.pos.png");

/// Window icon for the iced shell.
pub fn iced_icon() -> Option<iced::window::Icon> {
    let image = image::load_from_memory(ICON_PNG).ok()?.into_rgba8();
    let (width, height) = (image.width(), image.height());
    iced::window::icon::from_rgba(image.into_raw(), width, height).ok()
}

/// Reusable image handle for showing the app icon inside the iced UI.
pub fn iced_handle() -> iced::widget::image::Handle {
    static HANDLE: std::sync::OnceLock<iced::widget::image::Handle> = std::sync::OnceLock::new();
    HANDLE
        .get_or_init(|| iced::widget::image::Handle::from_bytes(ICON_PNG))
        .clone()
}
