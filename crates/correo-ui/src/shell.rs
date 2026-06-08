use correo_core::{sample_snapshot, AppCommandSender, AppSnapshot, ThemeMode, Workspace};
use egui::{CentralPanel, Frame, SidePanel, Stroke, TopBottomPanel};

use crate::{
    command_bar, connection_launcher, diagnostics, migration_recovery, nav, theme, workspace,
};

pub const THEME_KEY: &str = "correo.theme-mode";

pub struct CorreoUi {
    command_sender: AppCommandSender,
    snapshot: AppSnapshot,
}

impl CorreoUi {
    pub fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        let theme_mode = stored_theme(creation_context);
        let snapshot = sample_snapshot(theme_mode);
        theme::apply_theme(&creation_context.egui_ctx, theme_mode);
        Self::for_snapshot(snapshot)
    }

    pub fn for_snapshot(snapshot: AppSnapshot) -> Self {
        Self {
            command_sender: AppCommandSender::disconnected(),
            snapshot,
        }
    }

    pub fn for_snapshot_with_command_sender(
        snapshot: AppSnapshot,
        command_sender: AppCommandSender,
    ) -> Self {
        Self {
            command_sender,
            snapshot,
        }
    }

    pub fn with_command_sender(
        creation_context: &eframe::CreationContext<'_>,
        snapshot: AppSnapshot,
        command_sender: AppCommandSender,
    ) -> Self {
        theme::apply_theme(&creation_context.egui_ctx, snapshot.theme_mode);
        Self {
            command_sender,
            snapshot,
        }
    }

    pub fn for_theme(theme_mode: ThemeMode) -> Self {
        Self::for_snapshot(sample_snapshot(theme_mode))
    }

    pub fn set_snapshot(&mut self, snapshot: AppSnapshot) {
        self.snapshot = snapshot;
    }

    pub fn theme_mode(&self) -> ThemeMode {
        self.snapshot.theme_mode
    }

    pub fn draw(&mut self, context: &egui::Context) {
        let snapshot = self.snapshot.clone();
        theme::apply_theme(context, snapshot.theme_mode);
        let tokens = theme::tokens(context, snapshot.theme_mode);
        let commands = &self.command_sender;

        if snapshot.migration_recovery.blocks_normal_shell() {
            TopBottomPanel::top("correo-recovery-command")
                .exact_height(40.0)
                .frame(top_frame(tokens))
                .show(context, |ui| {
                    migration_recovery::top_bar(ui, &snapshot.migration_recovery, commands);
                });

            diagnostics_panel(&snapshot)
                .frame(top_frame(tokens))
                .show(context, |ui| {
                    diagnostics::strip(ui, &snapshot, tokens, commands);
                });

            SidePanel::left("correo-recovery-context")
                .default_width(280.0)
                .width_range(240.0..=360.0)
                .resizable(true)
                .frame(sidebar_frame(tokens))
                .show(context, |ui| {
                    migration_recovery::context_panel(ui, &snapshot.migration_recovery, tokens);
                });

            CentralPanel::default()
                .frame(central_frame(tokens))
                .show(context, |ui| {
                    migration_recovery::show(ui, &snapshot.migration_recovery, tokens, commands);
                });
            return;
        }

        TopBottomPanel::top("correo-menu")
            .exact_height(28.0)
            .frame(top_frame(tokens))
            .show(context, |ui| {
                command_bar::menu_bar(ui, commands);
            });

        TopBottomPanel::top("correo-command")
            .exact_height(40.0)
            .frame(top_frame(tokens))
            .show(context, |ui| {
                command_bar::command_bar(ui, &snapshot, tokens, commands);
            });

        diagnostics_panel(&snapshot)
            .frame(top_frame(tokens))
            .show(context, |ui| {
                diagnostics::strip(ui, &snapshot, tokens, commands);
            });

        SidePanel::left("correo-rail")
            .exact_width(48.0)
            .resizable(false)
            .frame(rail_frame(tokens))
            .show(context, |ui| {
                nav::rail(ui, snapshot.active_workspace, tokens, commands);
            });

        SidePanel::left("correo-context")
            .default_width(260.0)
            .width_range(220.0..=360.0)
            .resizable(true)
            .frame(sidebar_frame(tokens))
            .show(context, |ui| match snapshot.active_workspace {
                Workspace::Connections => {
                    connection_launcher::panel(ui, &snapshot, tokens, commands);
                }
                active_workspace => {
                    workspace::sidebar(ui, &snapshot, active_workspace, tokens, commands);
                }
            });

        CentralPanel::default()
            .frame(central_frame(tokens))
            .show(context, |ui| {
                workspace::show(ui, &snapshot, tokens, commands);
            });
    }
}

impl Default for CorreoUi {
    fn default() -> Self {
        Self::for_theme(ThemeMode::System)
    }
}

impl eframe::App for CorreoUi {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw(context);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, THEME_KEY, &self.snapshot.theme_mode);
    }
}

pub fn stored_theme(creation_context: &eframe::CreationContext<'_>) -> ThemeMode {
    creation_context
        .storage
        .and_then(|storage| eframe::get_value::<ThemeMode>(storage, THEME_KEY))
        .unwrap_or_default()
}

fn diagnostics_panel(snapshot: &AppSnapshot) -> TopBottomPanel {
    let panel = TopBottomPanel::bottom("correo-diagnostics");
    if snapshot.diagnostics_expanded {
        panel
            .default_height(220.0)
            .height_range(180.0..=320.0)
            .resizable(true)
    } else {
        panel.exact_height(28.0).resizable(false)
    }
}

fn top_frame(tokens: theme::ThemeTokens) -> Frame {
    Frame::NONE
        .fill(tokens.panel_bg)
        .stroke(Stroke::new(1.0, tokens.border))
        .inner_margin(egui::Margin::symmetric(8, 4))
}

fn rail_frame(tokens: theme::ThemeTokens) -> Frame {
    Frame::NONE
        .fill(tokens.panel_bg)
        .stroke(Stroke::new(1.0, tokens.border))
        .inner_margin(egui::Margin::same(4))
}

fn sidebar_frame(tokens: theme::ThemeTokens) -> Frame {
    Frame::NONE
        .fill(tokens.panel_bg)
        .stroke(Stroke::new(1.0, tokens.border))
        .inner_margin(egui::Margin::same(10))
}

fn central_frame(tokens: theme::ThemeTokens) -> Frame {
    Frame::NONE
        .fill(tokens.window_bg)
        .inner_margin(egui::Margin::same(12))
}
