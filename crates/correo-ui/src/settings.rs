use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, GlobalSettingField, GlobalSettingFlag,
    GlobalSettingsSnapshot, LegacyMigrationStatus, MigrationRecoveryCommand, SettingsFeedbackKind,
    SettingsOption, ThemeMode,
};
use egui::{Button, ComboBox, RichText, ScrollArea, TextEdit, Ui};

use crate::theme::ThemeTokens;

pub fn show(ui: &mut Ui, snapshot: &AppSnapshot, tokens: ThemeTokens, commands: &AppCommandSender) {
    let settings = &snapshot.global_settings;
    ui.horizontal(|ui| {
        ui.heading("Global Settings");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled(settings.dirty, Button::new("Save"))
                .clicked()
            {
                send(commands, AppCommand::SaveGlobalSettings);
            }
            if ui
                .add_enabled(settings.dirty, Button::new("Discard"))
                .clicked()
            {
                send(commands, AppCommand::DiscardGlobalSettings);
            }
        });
    });
    ui.separator();

    ScrollArea::vertical()
        .id_salt("global-settings-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            section(ui, "Appearance", tokens, |ui| {
                appearance(ui, snapshot.theme_mode, commands);
            });
            section(ui, "Language", tokens, |ui| {
                language(ui, settings, commands);
            });
            section(ui, "Search", tokens, |ui| {
                search(ui, settings, commands);
            });
            section(ui, "Keyring", tokens, |ui| {
                keyring(ui, settings, tokens, commands);
            });
            section(ui, "Updates", tokens, |ui| {
                updates(ui, settings, commands);
            });
            section(ui, "Plugins", tokens, |ui| {
                plugins(ui, settings, tokens, commands);
            });
            section(ui, "Data", tokens, |ui| {
                data(ui, settings, tokens, commands);
            });

            if let Some(feedback) = &settings.feedback {
                ui.separator();
                ui.label(
                    RichText::new(&feedback.message).color(feedback_color(feedback.kind, tokens)),
                );
            }
        });
}

fn section(ui: &mut Ui, title: &str, tokens: ThemeTokens, add: impl FnOnce(&mut Ui)) {
    ui.add_space(6.0);
    ui.label(
        RichText::new(title)
            .strong()
            .size(15.0)
            .color(tokens.text_primary),
    );
    ui.add_space(6.0);
    add(ui);
    ui.add_space(12.0);
    ui.separator();
}

fn appearance(ui: &mut Ui, theme_mode: ThemeMode, commands: &AppCommandSender) {
    row(ui, "Theme", |ui| {
        let mut selected = theme_mode;
        ComboBox::from_id_salt("settings-theme")
            .selected_text(theme_mode.label())
            .width(160.0)
            .show_ui(ui, |ui| {
                for mode in ThemeMode::ALL {
                    ui.selectable_value(&mut selected, mode, mode.label());
                }
            });
        if selected != theme_mode {
            send(commands, AppCommand::SetThemeMode(selected));
        }
    });
}

fn language(ui: &mut Ui, settings: &GlobalSettingsSnapshot, commands: &AppCommandSender) {
    row(ui, "Language", |ui| {
        option_combo(
            ui,
            "settings-language",
            &settings.language,
            &settings.language_options,
            |value| AppCommand::UpdateGlobalSetting {
                field: GlobalSettingField::Language,
                value,
            },
            commands,
        );
    });
}

fn search(ui: &mut Ui, settings: &GlobalSettingsSnapshot, commands: &AppCommandSender) {
    checkbox_flag(
        ui,
        "Use regular expressions",
        settings.search_use_regex,
        GlobalSettingFlag::UseRegexForSearch,
        commands,
    );
    checkbox_flag(
        ui,
        "Ignore case",
        settings.search_ignore_case,
        GlobalSettingFlag::UseIgnoreCase,
        commands,
    );
}

fn keyring(
    ui: &mut Ui,
    settings: &GlobalSettingsSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    row(ui, "Backend", |ui| {
        option_combo(
            ui,
            "settings-keyring",
            &settings.keyring_backend,
            &settings.keyring_options,
            |value| AppCommand::UpdateGlobalSetting {
                field: GlobalSettingField::KeyringBackend,
                value,
            },
            commands,
        );
    });
    ui.add_space(8.0);
    ui.label(RichText::new(&settings.cleanup_status).color(tokens.text_secondary));
    ui.add_enabled(false, Button::new("Delete sensitive data..."));
}

fn updates(ui: &mut Ui, settings: &GlobalSettingsSnapshot, commands: &AppCommandSender) {
    checkbox_flag(
        ui,
        "Check for updates",
        settings.update_checks_enabled,
        GlobalSettingFlag::SearchUpdates,
        commands,
    );
    ui.label(&settings.last_update_check);
}

fn plugins(
    ui: &mut Ui,
    settings: &GlobalSettingsSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    checkbox_flag(
        ui,
        "Use default repository",
        settings.use_default_plugin_repository,
        GlobalSettingFlag::UseDefaultPluginRepository,
        commands,
    );
    checkbox_flag(
        ui,
        "Install bundled plugins",
        settings.install_bundled_plugins,
        GlobalSettingFlag::InstallBundledPlugins,
        commands,
    );
    row(ui, "Bundled URL", |ui| {
        let mut url = settings.bundled_plugins_url.clone();
        let response = ui.add_sized([420.0, 24.0], TextEdit::singleline(&mut url));
        if response.changed() {
            send(
                commands,
                AppCommand::UpdateGlobalSetting {
                    field: GlobalSettingField::BundledPluginsUrl,
                    value: url,
                },
            );
        }
    });
    ui.separator();
    ui.label(RichText::new("Plugin repositories").strong());
    if settings.plugin_repositories.is_empty() {
        ui.label(RichText::new("No custom repositories").color(tokens.text_secondary));
    } else {
        for repository in &settings.plugin_repositories {
            ui.horizontal(|ui| {
                ui.monospace(&repository.id);
                ui.label(RichText::new(&repository.url).color(tokens.text_secondary));
            });
        }
    }
}

fn data(
    ui: &mut Ui,
    settings: &GlobalSettingsSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    value_row(ui, "Config version", &settings.config_version);
    value_row(ui, "Window", &settings.window_geometry);
    value_row(
        ui,
        "First start",
        if settings.first_start { "yes" } else { "no" },
    );
    ui.separator();
    legacy_migration(ui, settings, tokens, commands);
    ui.separator();
    ui.label(RichText::new(&settings.cleanup_status).color(tokens.text_secondary));
}

fn legacy_migration(
    ui: &mut Ui,
    settings: &GlobalSettingsSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let migration = &settings.legacy_migration;
    ui.label(RichText::new("Legacy migration").strong());
    value_row(ui, "Status", migration.status.label());
    value_row(ui, "Last result", &migration.last_status);
    if let Some(path) = &migration.legacy_path_hint {
        value_row(ui, "Legacy path", path);
    }
    if let Some(backup) = &migration.backup_name {
        value_row(ui, "Backup", backup);
    }
    if let Some(path) = &migration.backup_path_hint {
        value_row(ui, "Backup path", path);
    }
    if migration.warning_count > 0 {
        value_row(ui, "Warnings", &migration.warning_count.to_string());
    }
    ui.horizontal_wrapped(|ui| {
        if ui
            .add_enabled(
                migration.diagnostics_available,
                Button::new("View diagnostics"),
            )
            .clicked()
        {
            send(
                commands,
                AppCommand::MigrationRecovery(MigrationRecoveryCommand::OpenDiagnostics),
            );
        }
        if ui
            .add_enabled(
                migration.restore_available,
                Button::new("Restore backup..."),
            )
            .clicked()
        {
            send(
                commands,
                AppCommand::MigrationRecovery(MigrationRecoveryCommand::RequestRestoreBackup),
            );
        }
    });
    if migration.status == LegacyMigrationStatus::NotRun {
        ui.label(RichText::new("No restore target is available.").color(tokens.text_secondary));
    }
}

fn row(ui: &mut Ui, label: &str, add: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.set_min_height(30.0);
        ui.add_sized([160.0, 24.0], egui::Label::new(label));
        add(ui);
    });
}

fn value_row(ui: &mut Ui, label: &str, value: &str) {
    row(ui, label, |ui| {
        ui.label(value);
    });
}

fn checkbox_flag(
    ui: &mut Ui,
    label: &str,
    current: bool,
    flag: GlobalSettingFlag,
    commands: &AppCommandSender,
) {
    let mut enabled = current;
    if ui.checkbox(&mut enabled, label).changed() {
        send(commands, AppCommand::SetGlobalSettingFlag { flag, enabled });
    }
}

fn option_combo(
    ui: &mut Ui,
    id: &str,
    current: &str,
    options: &[SettingsOption],
    command: impl FnOnce(String) -> AppCommand,
    commands: &AppCommandSender,
) {
    let mut selected = current.to_owned();
    ComboBox::from_id_salt(id)
        .selected_text(option_label(current, options))
        .width(180.0)
        .show_ui(ui, |ui| {
            for option in options {
                ui.selectable_value(&mut selected, option.id.clone(), &option.label);
            }
        });
    if selected != current {
        send(commands, command(selected));
    }
}

fn option_label(current: &str, options: &[SettingsOption]) -> String {
    options
        .iter()
        .find(|option| option.id == current)
        .map(|option| option.label.clone())
        .unwrap_or_else(|| current.to_owned())
}

fn feedback_color(kind: SettingsFeedbackKind, tokens: ThemeTokens) -> egui::Color32 {
    match kind {
        SettingsFeedbackKind::Info => tokens.success,
        SettingsFeedbackKind::Warning => tokens.warning,
        SettingsFeedbackKind::Error => tokens.danger,
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
