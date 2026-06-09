use egui::{Align2, Hyperlink, RichText, Ui, Window};

use crate::theme::ThemeTokens;

const WEBSITE_URL: &str = env!("CARGO_PKG_REPOSITORY");

pub fn show(ui: &mut Ui, tokens: ThemeTokens) {
    Window::new("About CorreoMQTT")
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .show(ui.ctx(), |ui| about_dialog(ui, tokens));
}

fn about_dialog(ui: &mut Ui, tokens: ThemeTokens) {
    ui.set_width(360.0);
    ui.heading("CorreoMQTT");
    ui.label(RichText::new("Native Rust desktop port").color(tokens.text_secondary));
    ui.add_space(12.0);
    value_row(ui, "Version", env!("CARGO_PKG_VERSION"));
    value_row(ui, "License", env!("CARGO_PKG_LICENSE"));
    ui.add_space(12.0);
    ui.add(Hyperlink::from_label_and_url("Website", WEBSITE_URL));
}

fn value_row(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).strong());
        ui.label(value);
    });
}
