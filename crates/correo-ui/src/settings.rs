use correo_core::{
    AppCommand, AppCommandSender, AppSnapshot, GlobalSettingField, GlobalSettingFlag,
    GlobalSettingsSnapshot, LegacyMigrationStatus, MigrationRecoveryCommand, SettingsFeedbackKind,
    SettingsOption, ThemeMode,
};
use egui::{Button, ComboBox, RichText, ScrollArea, TextEdit, Ui};

use crate::i18n::I18n;
use crate::theme::ThemeTokens;

pub fn show(
    ui: &mut Ui,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let settings = &snapshot.global_settings;
    ui.horizontal(|ui| {
        ui.heading(i18n.text("settings-header"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled(settings.dirty, Button::new(i18n.text("common-save")))
                .clicked()
            {
                send(commands, AppCommand::SaveGlobalSettings);
            }
            if ui
                .add_enabled(settings.dirty, Button::new(i18n.text("common-discard")))
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
            section(
                ui,
                i18n.settings_section_label(correo_core::SettingsSection::Appearance),
                tokens,
                |ui| {
                    appearance(ui, snapshot.theme_mode, commands, i18n);
                },
            );
            section(
                ui,
                i18n.settings_section_label(correo_core::SettingsSection::Language),
                tokens,
                |ui| {
                    language(ui, settings, commands, i18n);
                },
            );
            section(
                ui,
                i18n.settings_section_label(correo_core::SettingsSection::Search),
                tokens,
                |ui| {
                    search(ui, settings, commands, i18n);
                },
            );
            section(
                ui,
                i18n.settings_section_label(correo_core::SettingsSection::Keyring),
                tokens,
                |ui| {
                    keyring(ui, settings, tokens, commands, i18n);
                },
            );
            section(
                ui,
                i18n.settings_section_label(correo_core::SettingsSection::Updates),
                tokens,
                |ui| {
                    updates(ui, settings, commands, i18n);
                },
            );
            section(
                ui,
                i18n.settings_section_label(correo_core::SettingsSection::Plugins),
                tokens,
                |ui| {
                    plugins(ui, settings, tokens, commands, i18n);
                },
            );
            section(
                ui,
                i18n.settings_section_label(correo_core::SettingsSection::Data),
                tokens,
                |ui| {
                    data(ui, settings, tokens, commands, i18n);
                },
            );

            if let Some(feedback) = &settings.feedback {
                ui.separator();
                ui.label(
                    RichText::new(&feedback.message).color(feedback_color(feedback.kind, tokens)),
                );
            }
        });
}

fn section(ui: &mut Ui, title: String, tokens: ThemeTokens, add: impl FnOnce(&mut Ui)) {
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

fn appearance(ui: &mut Ui, theme_mode: ThemeMode, commands: &AppCommandSender, i18n: &I18n) {
    row(ui, &i18n.text("settings-theme"), |ui| {
        let mut selected = theme_mode;
        ComboBox::from_id_salt("settings-theme")
            .selected_text(i18n.theme_label(theme_mode))
            .width(160.0)
            .show_ui(ui, |ui| {
                for mode in ThemeMode::ALL {
                    ui.selectable_value(&mut selected, mode, i18n.theme_label(mode));
                }
            });
        if selected != theme_mode {
            send(commands, AppCommand::SetThemeMode(selected));
        }
    });
}

fn language(
    ui: &mut Ui,
    settings: &GlobalSettingsSnapshot,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    row(ui, &i18n.text("settings-language"), |ui| {
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
            i18n,
        );
    });
}

fn search(
    ui: &mut Ui,
    settings: &GlobalSettingsSnapshot,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    checkbox_flag(
        ui,
        &i18n.text("settings-use-regex"),
        settings.search_use_regex,
        GlobalSettingFlag::UseRegexForSearch,
        commands,
    );
    checkbox_flag(
        ui,
        &i18n.text("settings-ignore-case"),
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
    i18n: &I18n,
) {
    row(ui, &i18n.text("settings-backend"), |ui| {
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
            i18n,
        );
    });
    ui.add_space(8.0);
    ui.label(RichText::new(&settings.cleanup_status).color(tokens.text_secondary));
    ui.add_enabled(
        false,
        Button::new(i18n.text("settings-delete-sensitive-data")),
    );
}

fn updates(
    ui: &mut Ui,
    settings: &GlobalSettingsSnapshot,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    checkbox_flag(
        ui,
        &i18n.text("settings-updates"),
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
    i18n: &I18n,
) {
    checkbox_flag(
        ui,
        &i18n.text("settings-use-default-repository"),
        settings.use_default_plugin_repository,
        GlobalSettingFlag::UseDefaultPluginRepository,
        commands,
    );
    checkbox_flag(
        ui,
        &i18n.text("settings-install-bundled-plugins"),
        settings.install_bundled_plugins,
        GlobalSettingFlag::InstallBundledPlugins,
        commands,
    );
    row(ui, &i18n.text("settings-bundled-url"), |ui| {
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
    ui.label(RichText::new(i18n.text("settings-plugin-repositories")).strong());
    if settings.plugin_repositories.is_empty() {
        ui.label(
            RichText::new(i18n.text("settings-no-custom-repositories"))
                .color(tokens.text_secondary),
        );
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
    i18n: &I18n,
) {
    value_row(
        ui,
        &i18n.text("settings-config-version"),
        &settings.config_version,
    );
    value_row(ui, &i18n.text("settings-window"), &settings.window_geometry);
    value_row(
        ui,
        &i18n.text("settings-first-start"),
        if settings.first_start { "yes" } else { "no" },
    );
    ui.separator();
    legacy_migration(ui, settings, tokens, commands, i18n);
    ui.separator();
    ui.label(RichText::new(&settings.cleanup_status).color(tokens.text_secondary));
}

fn legacy_migration(
    ui: &mut Ui,
    settings: &GlobalSettingsSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let migration = &settings.legacy_migration;
    ui.label(RichText::new(i18n.text("settings-legacy-migration")).strong());
    value_row(
        ui,
        &i18n.text("settings-status"),
        &i18n.legacy_migration_label(migration.status),
    );
    value_row(
        ui,
        &i18n.text("settings-last-result"),
        &migration.last_status,
    );
    if let Some(path) = &migration.legacy_path_hint {
        value_row(ui, &i18n.text("settings-legacy-path"), path);
    }
    if let Some(backup) = &migration.backup_name {
        value_row(ui, &i18n.text("settings-backup"), backup);
    }
    if let Some(path) = &migration.backup_path_hint {
        value_row(ui, &i18n.text("settings-backup-path"), path);
    }
    if migration.warning_count > 0 {
        value_row(
            ui,
            &i18n.text("settings-warnings"),
            &migration.warning_count.to_string(),
        );
    }
    ui.horizontal_wrapped(|ui| {
        if ui
            .add_enabled(
                migration.diagnostics_available,
                Button::new(i18n.text("settings-view-diagnostics")),
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
                Button::new(i18n.text("settings-restore-backup")),
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
        ui.label(
            RichText::new(i18n.text("settings-no-restore-target")).color(tokens.text_secondary),
        );
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
    i18n: &I18n,
) {
    let mut selected = current.to_owned();
    ComboBox::from_id_salt(id)
        .selected_text(option_label(current, options, i18n))
        .width(180.0)
        .show_ui(ui, |ui| {
            for option in options {
                let label = i18n.language_option_label(&option.id, &option.label);
                ui.selectable_value(&mut selected, option.id.clone(), label);
            }
        });
    if selected != current {
        send(commands, command(selected));
    }
}

fn option_label(current: &str, options: &[SettingsOption], i18n: &I18n) -> String {
    options
        .iter()
        .find(|option| option.id == current)
        .map(|option| i18n.language_option_label(&option.id, &option.label))
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
