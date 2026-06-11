use correo_core::{AppCommand, AppCommandSender, TransferConnectionRow, TransferConnectionStatus};
use egui::{RichText, ScrollArea, Ui};
use egui_phosphor::regular;

use crate::i18n::I18n;
use crate::theme::ThemeTokens;
use crate::widgets::checkbox;

const LIST_WIDTH: f32 = 260.0;

pub(crate) fn connection_list(
    ui: &mut Ui,
    rows: &[TransferConnectionRow],
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    import: bool,
) {
    ScrollArea::vertical()
        .id_salt(if import {
            "import-connection-list"
        } else {
            "export-connection-list"
        })
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_width(LIST_WIDTH - 18.0);
            for row in rows {
                connection_row(ui, row, tokens, commands, i18n, import);
            }
        });
}

pub(crate) fn select_buttons(
    ui: &mut Ui,
    rows: &[TransferConnectionRow],
    commands: &AppCommandSender,
    i18n: &I18n,
    import: bool,
) {
    ui.horizontal(|ui| {
        if ui
            .button(format!(
                "{} {}",
                regular::CHECK_SQUARE,
                i18n.text("common-select-all")
            ))
            .clicked()
        {
            for row in rows.iter().filter(|row| !import || importable(row)) {
                send_select(commands, row.id.clone(), true, import);
            }
        }
        if ui
            .button(format!(
                "{} {}",
                regular::SQUARE,
                i18n.text("common-select-none")
            ))
            .clicked()
        {
            for row in rows {
                send_select(commands, row.id.clone(), false, import);
            }
        }
    });
}

fn connection_row(
    ui: &mut Ui,
    row: &TransferConnectionRow,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
    import: bool,
) {
    let enabled = !import || importable(row);
    ui.add_enabled_ui(enabled, |ui| {
        ui.horizontal(|ui| {
            let mut selected = row.selected && enabled;
            if checkbox(ui, &mut selected, "").changed() {
                send_select(commands, row.id.clone(), selected, import);
            }
            ui.vertical(|ui| {
                let name_color = if enabled {
                    tokens.text_primary
                } else {
                    tokens.text_disabled
                };
                ui.label(RichText::new(&row.name).strong().color(name_color));
                let detail = if enabled {
                    row.endpoint.clone()
                } else {
                    i18n.text("transfer-import-exists-already")
                };
                ui.label(RichText::new(detail).color(tokens.text_secondary));
            });
        });
    });
    ui.add_space(8.0);
}

fn send_select(commands: &AppCommandSender, row_id: String, selected: bool, import: bool) {
    let command = if import {
        AppCommand::SelectConnectionImportRow { row_id, selected }
    } else {
        AppCommand::SelectConnectionExportRow { row_id, selected }
    };
    let _ = commands.send(command);
}

fn importable(row: &TransferConnectionRow) -> bool {
    matches!(
        row.status,
        TransferConnectionStatus::New | TransferConnectionStatus::MissingSecret
    )
}
