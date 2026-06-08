use correo_core::{DiagnosticSeverity, ThemeMode};
use egui::{Color32, CornerRadius, Stroke, Theme, ThemePreference, Visuals};

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
        style.spacing.button_padding = egui::vec2(8.0, 4.0);
        style.spacing.menu_margin = egui::Margin::same(8);
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
        window_bg: rgb(0x11, 0x14, 0x18),
        panel_bg: rgb(0x18, 0x1D, 0x22),
        panel_raised: rgb(0x20, 0x26, 0x2D),
        field_bg: rgb(0x0F, 0x12, 0x15),
        border: rgb(0x33, 0x3C, 0x45),
        text_primary: rgb(0xE7, 0xED, 0xF3),
        text_secondary: rgb(0x9A, 0xA6, 0xB2),
        text_disabled: rgb(0x5E, 0x68, 0x72),
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
