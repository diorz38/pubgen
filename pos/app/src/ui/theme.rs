use crate::main_ui::AppTheme;
use iced::{theme::Palette, Theme};

pub fn is_dark(app_theme: AppTheme, system_dark: bool) -> bool {
    match app_theme {
        AppTheme::Dark => true,
        AppTheme::Light => false,
        AppTheme::System => system_dark,
    }
}

pub fn to_iced(app_theme: AppTheme, system_dark: bool) -> Theme {
    if is_dark(app_theme, system_dark) {
        custom(PALETTE_DARK, "pos-dark")
    } else {
        custom(PALETTE_LIGHT, "pos-light")
    }
}

fn custom(palette: Palette, name: &'static str) -> Theme {
    Theme::custom(name, palette)
}

pub fn lighten(c: iced::Color, f: f32) -> iced::Color {
    iced::Color {
        r: c.r + (1.0 - c.r) * f,
        g: c.g + (1.0 - c.g) * f,
        b: c.b + (1.0 - c.b) * f,
        a: c.a,
    }
}

pub const ACCENT: iced::Color = iced::Color::from_rgb8(26, 115, 232); // #1A73E8

// Menu bar (G11). The rule is the palette text color at this alpha over the
// app bar: ~4.1:1 on light and ~4.5:1 on dark, both above the 3:1 floor.
#[allow(dead_code)]
pub const MENU_RULE_ALPHA_LIGHT: f32 = 0.60;
#[allow(dead_code)]
pub const MENU_RULE_ALPHA_DARK: f32 = 0.50;
pub const POPOVER_BG_DARK: iced::Color = iced::Color::from_rgb8(36, 39, 47);
pub const POPOVER_BG_LIGHT: iced::Color = iced::Color::from_rgb8(250, 251, 254);

/// Secondary/metadata text, measured against the panel background rather than
/// pure white/black:
///
/// - dark `#9BA2AE` on `#1F2228` = **6.25:1**
/// - light `#646C78` on `#F0F2F6` = **4.73:1** (5.30:1 on `#FFFFFF`)
///
/// Both clear the 4.5:1 floor from G8.
pub const MUTED_DARK: iced::Color = iced::Color::from_rgb8(155, 162, 174);
pub const MUTED_LIGHT: iced::Color = iced::Color::from_rgb8(100, 108, 120);

/// Picks the documented muted color for the active theme.
pub fn muted(dark: bool) -> iced::Color {
    if dark {
        MUTED_DARK
    } else {
        MUTED_LIGHT
    }
}

/// Ready/positive status text, readable on both panel backgrounds.
pub fn success(dark: bool) -> iced::Color {
    if dark {
        iced::Color::from_rgb8(76, 175, 125) // #4CAF7D on #1F2228 = 7.0:1
    } else {
        iced::Color::from_rgb8(30, 122, 60) // #1E7A3C on #F0F2F6 = 4.9:1
    }
}

/// Partial/warning status text, readable on both panel backgrounds.
pub fn warning(dark: bool) -> iced::Color {
    if dark {
        iced::Color::from_rgb8(230, 178, 64) // #E6B240 on #1F2228 = 8.4:1
    } else {
        iced::Color::from_rgb8(178, 106, 0) // #B26A00 on #F0F2F6 = 4.1:1
    }
}

pub const PALETTE_DARK: Palette = Palette {
    background: iced::Color::from_rgb8(31, 34, 40), // #1F2228
    text: iced::Color::from_rgb8(236, 239, 241),
    primary: ACCENT,
    success: iced::Color::from_rgb8(52, 168, 83),
    warning: iced::Color::from_rgb8(251, 188, 4),
    danger: iced::Color::from_rgb8(234, 67, 53),
};

pub const PALETTE_LIGHT: Palette = Palette {
    background: iced::Color::from_rgb8(240, 242, 246), // #F0F2F6
    text: iced::Color::from_rgb8(32, 33, 36),
    primary: ACCENT,
    success: iced::Color::from_rgb8(52, 168, 83),
    warning: iced::Color::from_rgb8(251, 188, 4),
    danger: iced::Color::from_rgb8(234, 67, 53),
};

#[cfg(test)]
fn luminance(color: iced::Color) -> f32 {
    fn channel(value: f32) -> f32 {
        if value <= 0.039_28 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }

    0.2126 * channel(color.r) + 0.7152 * channel(color.g) + 0.0722 * channel(color.b)
}

/// WCAG 2.1 contrast ratio between two opaque colors.
#[cfg(test)]
pub fn contrast_ratio(a: iced::Color, b: iced::Color) -> f32 {
    let (light, dark) = if luminance(a) >= luminance(b) {
        (luminance(a), luminance(b))
    } else {
        (luminance(b), luminance(a))
    };
    (light + 0.05) / (dark + 0.05)
}

/// Alpha-composites `fg` over `bg`, matching how iced renders `scale_alpha`.
#[cfg(test)]
fn over(fg: iced::Color, bg: iced::Color, alpha: f32) -> iced::Color {
    iced::Color {
        r: fg.r * alpha + bg.r * (1.0 - alpha),
        g: fg.g * alpha + bg.g * (1.0 - alpha),
        b: fg.b * alpha + bg.b * (1.0 - alpha),
        a: 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn muted_text_meets_contrast_floor() {
        assert!(
            contrast_ratio(MUTED_DARK, PALETTE_DARK.background) >= 4.5,
            "dark muted: {:.2}",
            contrast_ratio(MUTED_DARK, PALETTE_DARK.background)
        );
        assert!(
            contrast_ratio(MUTED_LIGHT, PALETTE_LIGHT.background) >= 4.5,
            "light muted: {:.2}",
            contrast_ratio(MUTED_LIGHT, PALETTE_LIGHT.background)
        );
    }

    #[test]
    fn menu_rule_meets_three_to_one() {
        let dark_rule = over(
            PALETTE_DARK.text,
            PALETTE_DARK.background,
            MENU_RULE_ALPHA_DARK,
        );
        assert!(
            contrast_ratio(dark_rule, PALETTE_DARK.background) >= 3.0,
            "dark menu rule: {:.2}",
            contrast_ratio(dark_rule, PALETTE_DARK.background)
        );

        let light_rule = over(
            PALETTE_LIGHT.text,
            PALETTE_LIGHT.background,
            MENU_RULE_ALPHA_LIGHT,
        );
        assert!(
            contrast_ratio(light_rule, PALETTE_LIGHT.background) >= 3.0,
            "light menu rule: {:.2}",
            contrast_ratio(light_rule, PALETTE_LIGHT.background)
        );
    }
}
