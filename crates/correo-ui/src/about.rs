use egui::{Align2, Hyperlink, RichText, Ui, Window};

use crate::i18n::I18n;
use crate::theme::ThemeTokens;

const WEBSITE_URL: &str = env!("CARGO_PKG_REPOSITORY");

pub fn show(ui: &mut Ui, tokens: ThemeTokens, i18n: &I18n) {
    Window::new(i18n.text("about-title"))
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .resizable(false)
        .show(ui.ctx(), |ui| about_dialog(ui, tokens, i18n));
}

fn about_dialog(ui: &mut Ui, tokens: ThemeTokens, i18n: &I18n) {
    ui.set_width(360.0);
    ui.heading("CorreoMQTT");
    ui.label(RichText::new(i18n.text("about-port")).color(tokens.text_secondary));
    ui.add_space(12.0);
    value_row(ui, &i18n.text("about-version"), env!("CARGO_PKG_VERSION"));
    value_row(ui, &i18n.text("about-license"), env!("CARGO_PKG_LICENSE"));
    ui.add_space(12.0);
    ui.add(Hyperlink::from_label_and_url(
        i18n.text("about-website"),
        WEBSITE_URL,
    ));
}

fn value_row(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).strong());
        ui.label(value);
    });
}
