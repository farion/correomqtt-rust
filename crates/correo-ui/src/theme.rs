use correo_core::{DiagnosticSeverity, ThemeMode};
use egui::{Color32, CornerRadius, Stroke, Theme, ThemePreference, Visuals};

pub const CONTROL_PADDING: i8 = 8;
pub const CONTROL_HEIGHT: f32 = 34.0;

#[derive(Debug, Clone, Copy)]
pub struct ThemeTokens {
    pub window_bg: Color32,
    pub panel_bg: Color32,
    pub panel_raised: Color32,
    pub field_bg: Color32,
    pub border: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_disabled: Color32,
    pub accent: Color32,
    pub accent_selected_bg: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
    pub script: Color32,
}

impl ThemeTokens {
    pub fn severity(self, severity: DiagnosticSeverity) -> Color32 {
        match severity {
            DiagnosticSeverity::Info => self.accent,
            DiagnosticSeverity::Warning => self.warning,
            DiagnosticSeverity::Error => self.danger,
        }
    }
}

pub fn apply_theme(ctx: &egui::Context, mode: ThemeMode) {
    ctx.set_visuals_of(Theme::Dark, visuals_for(dark_tokens(), true));
    ctx.set_visuals_of(Theme::Light, visuals_for(light_tokens(), false));
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = control_padding();
        style.spacing.interact_size.y = style.spacing.interact_size.y.max(CONTROL_HEIGHT);
        style.spacing.menu_margin = control_margin();
        style.visuals.window_corner_radius = CornerRadius::same(4);
        for widget in [
            &mut style.visuals.widgets.noninteractive,
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
            &mut style.visuals.widgets.open,
        ] {
            widget.corner_radius = CornerRadius::same(4);
        }
    });
    ctx.set_theme(match mode {
        ThemeMode::System => ThemePreference::System,
        ThemeMode::Light => ThemePreference::Light,
        ThemeMode::Dark => ThemePreference::Dark,
    });
}

pub fn control_margin() -> egui::Margin {
    egui::Margin::same(CONTROL_PADDING)
}

pub fn control_padding() -> egui::Vec2 {
    egui::vec2(CONTROL_PADDING as f32, CONTROL_PADDING as f32)
}

pub fn tokens(ctx: &egui::Context, mode: ThemeMode) -> ThemeTokens {
    match resolved_theme(ctx, mode) {
        Theme::Dark => dark_tokens(),
        Theme::Light => light_tokens(),
    }
}

pub fn static_tokens(mode: ThemeMode) -> ThemeTokens {
    match mode {
        ThemeMode::Light => light_tokens(),
        ThemeMode::System | ThemeMode::Dark => dark_tokens(),
    }
}

fn resolved_theme(ctx: &egui::Context, mode: ThemeMode) -> Theme {
    match mode {
        ThemeMode::Dark => Theme::Dark,
        ThemeMode::Light => Theme::Light,
        ThemeMode::System => ctx.system_theme().unwrap_or(Theme::Dark),
    }
}

fn visuals_for(tokens: ThemeTokens, dark_mode: bool) -> Visuals {
    let mut visuals = if dark_mode {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    visuals.dark_mode = dark_mode;
    visuals.window_fill = tokens.window_bg;
    visuals.panel_fill = tokens.panel_bg;
    visuals.extreme_bg_color = tokens.field_bg;
    visuals.faint_bg_color = tokens.panel_raised;
    visuals.code_bg_color = tokens.field_bg;
    visuals.override_text_color = Some(tokens.text_primary);
    visuals.warn_fg_color = tokens.warning;
    visuals.error_fg_color = tokens.danger;
    visuals.hyperlink_color = tokens.accent;
    visuals.selection.bg_fill = tokens.accent_selected_bg;
    visuals.selection.stroke = Stroke::new(1.0, tokens.accent);
    visuals.window_stroke = Stroke::new(1.0, tokens.border);
    visuals.widgets.noninteractive.bg_fill = tokens.panel_bg;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, tokens.text_primary);
    visuals.widgets.inactive.bg_fill = tokens.panel_raised;
    visuals.widgets.inactive.weak_bg_fill = tokens.panel_raised;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, tokens.border);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, tokens.text_primary);
    visuals.widgets.hovered.bg_fill = tokens.accent_selected_bg;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, tokens.accent);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, tokens.text_primary);
    visuals.widgets.active.bg_fill = tokens.accent_selected_bg;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, tokens.accent);
    visuals.widgets.open.bg_fill = tokens.panel_raised;
    visuals
}

fn dark_tokens() -> ThemeTokens {
    ThemeTokens {
        window_bg: gray(0x12),
        panel_bg: gray(0x1B),
        panel_raised: gray(0x26),
        field_bg: gray(0x10),
        border: gray(0x42),
        text_primary: gray(0xEE),
        text_secondary: gray(0xB0),
        text_disabled: gray(0x72),
        accent: rgb(0x2E, 0x8F, 0xCA),
        accent_selected_bg: rgb(0x17, 0x38, 0x4D),
        success: rgb(0x3F, 0xB9, 0x74),
        warning: rgb(0xE4, 0xA3, 0x43),
        danger: rgb(0xD9, 0x5C, 0x5C),
        script: rgb(0x8F, 0xBF, 0x4D),
    }
}

fn light_tokens() -> ThemeTokens {
    ThemeTokens {
        window_bg: rgb(0xF5, 0xF7, 0xFA),
        panel_bg: rgb(0xFF, 0xFF, 0xFF),
        panel_raised: rgb(0xED, 0xF1, 0xF5),
        field_bg: rgb(0xFF, 0xFF, 0xFF),
        border: rgb(0xD7, 0xDE, 0xE6),
        text_primary: rgb(0x17, 0x20, 0x2A),
        text_secondary: rgb(0x5F, 0x6B, 0x77),
        text_disabled: rgb(0x9A, 0xA6, 0xB2),
        accent: rgb(0x17, 0x6E, 0xA3),
        accent_selected_bg: rgb(0xDC, 0xEE, 0xFF),
        success: rgb(0x1F, 0x8E, 0x58),
        warning: rgb(0xA9, 0x64, 0x00),
        danger: rgb(0xB3, 0x3A, 0x3A),
        script: rgb(0x4F, 0x7E, 0x23),
    }
}

fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

fn gray(value: u8) -> Color32 {
    Color32::from_gray(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_neutral_tokens_are_grayscale() {
        let tokens = dark_tokens();
        for color in [
            tokens.window_bg,
            tokens.panel_bg,
            tokens.panel_raised,
            tokens.field_bg,
            tokens.border,
            tokens.text_primary,
            tokens.text_secondary,
            tokens.text_disabled,
        ] {
            let [r, g, b, _] = color.to_array();
            assert_eq!(r, g);
            assert_eq!(g, b);
        }
    }

    #[test]
    fn global_control_padding_is_eight_pixels() {
        assert_eq!(control_margin(), egui::Margin::same(8));
        assert_eq!(control_padding(), egui::vec2(8.0, 8.0));
    }
}
