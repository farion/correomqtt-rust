use egui::{Grid, Hyperlink, Image, RichText, ScrollArea, Ui};

use crate::i18n::I18n;
use crate::theme::ThemeTokens;

const WEBSITE_URL: &str = env!("CARGO_PKG_REPOSITORY");
const EXXETA_URL: &str = "https://exxeta.com";
const ABOUT_ICON_SIZE: f32 = 160.0;

mod build_info {
    include!(concat!(env!("OUT_DIR"), "/about_metadata.rs"));
}

pub fn show(ui: &mut Ui, _tokens: ThemeTokens, i18n: &I18n) {
    ScrollArea::vertical()
        .id_salt("about-content")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add(
                Image::new(egui::include_image!("../../../assets/icon.svg"))
                    .fit_to_exact_size(egui::Vec2::splat(ABOUT_ICON_SIZE)),
            );
            ui.add_space(18.0);
            value_row(
                ui,
                "CorreoMQTT",
                &format!("v{}", build_info::APP_VERSION.trim_start_matches('v')),
            );
            value_row(ui, &i18n.text("about-license"), env!("CARGO_PKG_LICENSE"));
            ui.add_space(12.0);
            ui.add(Hyperlink::from_label_and_url(
                i18n.text("about-website"),
                WEBSITE_URL,
            ));
            ui.add_space(18.0);
            ui.heading(i18n.text("about-contributors"));
            ui.add(Hyperlink::from_label_and_url("Exxeta", EXXETA_URL));
            ui.add_space(18.0);
            ui.heading(i18n.text("about-open-source-libraries"));
            ui.add_space(8.0);
            open_source_libraries(ui);
        });
}

fn value_row(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).strong());
        ui.label(value);
    });
}

fn open_source_libraries(ui: &mut Ui) {
    Grid::new("about-open-source-libraries")
        .num_columns(2)
        .min_row_height(0.0)
        .spacing([18.0, 0.0])
        .show(ui, |ui| {
            for (name, version) in build_info::OPEN_SOURCE_LIBRARIES {
                ui.label(RichText::new(*name).strong().size(14.0));
                ui.label(RichText::new(*version).size(14.0));
                ui.end_row();
            }
        });
}
