use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, GlobalSettingField, SettingsOption, ThemeMode,
};
use correo_style::layout;
use egui::{Align, ComboBox, Image, Layout, RichText, Ui};

use crate::i18n::I18n;
use crate::theme::ThemeTokens;

pub fn command_bar(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    _tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.horizontal_centered(|ui| {
        ui.add(
            Image::new(egui::include_image!("../../../assets/icon.svg")).fit_to_exact_size(
                egui::vec2(layout::HEADER_ICON_SIZE, layout::HEADER_ICON_SIZE),
            ),
        );
        ui.label(
            RichText::new("CorreoMQTT")
                .strong()
                .size(layout::APP_TITLE_SIZE),
        );

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            theme_selector(ui, &snapshot.theme_mode, commands, i18n);
            language_selector(
                ui,
                &snapshot.global_settings.language,
                &snapshot.global_settings.language_options,
                commands,
                i18n,
            );
        });
    });
}

fn theme_selector(ui: &mut Ui, current: &ThemeMode, commands: &AppCommandSender, i18n: &I18n) {
    let mut selected = current.clone();
    ComboBox::from_id_salt("theme-mode")
        .selected_text(i18n.theme_label(current))
        .width(layout::HEADER_THEME_SELECTOR_WIDTH)
        .show_ui(ui, |ui| {
            for mode in ThemeMode::ALL {
                ui.selectable_value(&mut selected, mode.clone(), i18n.theme_label(&mode));
            }
        });
    if selected != *current {
        let _ = commands.send(AppCommand::SetThemeMode(selected));
        let _ = commands.send(AppCommand::SaveGlobalSettings);
    }
}

fn language_selector(
    ui: &mut Ui,
    current: &str,
    options: &[SettingsOption],
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let mut selected = current.to_owned();
    ComboBox::from_id_salt("header-language")
        .selected_text(language_label(current, options, i18n))
        .width(layout::HEADER_LANGUAGE_SELECTOR_WIDTH)
        .show_ui(ui, |ui| {
            for option in options {
                let label = i18n.language_option_label(&option.id, &option.label);
                ui.selectable_value(&mut selected, option.id.clone(), label);
            }
        });
    if selected != current {
        let _ = commands.send(AppCommand::UpdateGlobalSetting {
            field: GlobalSettingField::Language,
            value: selected,
        });
        let _ = commands.send(AppCommand::SaveGlobalSettings);
    }
}

fn language_label(current: &str, options: &[SettingsOption], i18n: &I18n) -> String {
    options
        .iter()
        .find(|option| option.id == current)
        .map(|option| i18n.language_option_label(&option.id, &option.label))
        .unwrap_or_else(|| current.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_label_uses_available_option_labels() {
        let options = vec![
            SettingsOption {
                id: "system".to_owned(),
                label: "System".to_owned(),
            },
            SettingsOption {
                id: "de_DE".to_owned(),
                label: "Deutsch".to_owned(),
            },
        ];
        let i18n = I18n::new("en_US");

        assert_eq!(language_label("system", &options, &i18n), "System");
        assert_eq!(language_label("de_DE", &options, &i18n), "Deutsch");
        assert_eq!(language_label("custom", &options, &i18n), "custom");
    }

    #[test]
    fn app_title_font_size_is_scaled_by_one_and_a_half() {
        assert_eq!(layout::APP_TITLE_SIZE, 24.0);
        assert_eq!(layout::APP_TITLE_SIZE, layout::APP_TITLE_BASE_SIZE * 1.5);
    }
}
