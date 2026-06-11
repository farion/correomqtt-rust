use correo_core::{sample_snapshot, AppCommandSender, AppSnapshot, ThemeMode, Workspace};
use correo_style::{apply_theme, layout, tokens, ThemeTokens};
use egui::{CentralPanel, Frame, SidePanel, TopBottomPanel};

use crate::{
    command_bar, connection_launcher, i18n::I18n, icons, migration_recovery, nav, toasts,
    transfer_wizard, workspace,
};

pub const THEME_KEY: &str = "correo.theme-mode";

pub struct CorreoUi {
    command_sender: AppCommandSender,
    snapshot: AppSnapshot,
    i18n: I18n,
    icons_installed: bool,
    transfer_wizard: transfer_wizard::State,
}

impl CorreoUi {
    pub fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        let theme_mode = stored_theme(creation_context);
        let snapshot = sample_snapshot(theme_mode.clone());
        apply_theme(&creation_context.egui_ctx, &theme_mode);
        egui_extras::install_image_loaders(&creation_context.egui_ctx);
        icons::install(&creation_context.egui_ctx);
        Self {
            command_sender: AppCommandSender::disconnected(),
            i18n: I18n::new(&snapshot.global_settings.language),
            snapshot,
            icons_installed: true,
            transfer_wizard: transfer_wizard::State::default(),
        }
    }

    pub fn for_snapshot(snapshot: AppSnapshot) -> Self {
        Self {
            command_sender: AppCommandSender::disconnected(),
            i18n: I18n::new(&snapshot.global_settings.language),
            snapshot,
            icons_installed: false,
            transfer_wizard: transfer_wizard::State::default(),
        }
    }

    pub fn for_snapshot_with_command_sender(
        snapshot: AppSnapshot,
        command_sender: AppCommandSender,
    ) -> Self {
        Self {
            command_sender,
            i18n: I18n::new(&snapshot.global_settings.language),
            snapshot,
            icons_installed: false,
            transfer_wizard: transfer_wizard::State::default(),
        }
    }

    pub fn with_command_sender(
        creation_context: &eframe::CreationContext<'_>,
        snapshot: AppSnapshot,
        command_sender: AppCommandSender,
    ) -> Self {
        apply_theme(&creation_context.egui_ctx, &snapshot.theme_mode);
        egui_extras::install_image_loaders(&creation_context.egui_ctx);
        icons::install(&creation_context.egui_ctx);
        Self {
            command_sender,
            i18n: I18n::new(&snapshot.global_settings.language),
            snapshot,
            icons_installed: true,
            transfer_wizard: transfer_wizard::State::default(),
        }
    }

    pub fn for_theme(theme_mode: ThemeMode) -> Self {
        Self::for_snapshot(sample_snapshot(theme_mode))
    }

    pub fn set_snapshot(&mut self, snapshot: AppSnapshot) {
        self.i18n.set_language(&snapshot.global_settings.language);
        self.snapshot = snapshot;
    }

    pub fn theme_mode(&self) -> ThemeMode {
        self.snapshot.theme_mode.clone()
    }

    pub fn draw(&mut self, context: &egui::Context) {
        self.ensure_icons_installed(context);
        let snapshot = self.snapshot.clone();
        apply_theme(context, &snapshot.theme_mode);
        let tokens = tokens(context, &snapshot.theme_mode);
        let commands = &self.command_sender;
        let i18n = &self.i18n;

        if snapshot.migration_recovery.blocks_normal_shell() {
            TopBottomPanel::top("correo-recovery-command")
                .exact_height(layout::HEADER_HEIGHT)
                .frame(top_frame(tokens))
                .show(context, |ui| {
                    migration_recovery::top_bar(ui, &snapshot.migration_recovery);
                });

            SidePanel::left("correo-recovery-context")
                .default_width(layout::RECOVERY_CONTEXT_DEFAULT_WIDTH)
                .width_range(
                    layout::RECOVERY_CONTEXT_MIN_WIDTH..=layout::RECOVERY_CONTEXT_MAX_WIDTH,
                )
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

        TopBottomPanel::top("correo-command")
            .exact_height(layout::HEADER_HEIGHT)
            .frame(top_frame(tokens))
            .show(context, |ui| {
                command_bar::command_bar(ui, &snapshot, tokens, commands, i18n);
            });

        SidePanel::left("correo-rail")
            .exact_width(layout::RAIL_WIDTH)
            .resizable(false)
            .frame(rail_frame(tokens))
            .show(context, |ui| {
                nav::rail(ui, snapshot.active_workspace, tokens, commands, i18n);
            });

        if context_panel_visible(&snapshot) {
            SidePanel::left("correo-context")
                .default_width(layout::SIDEBAR_DEFAULT_WIDTH)
                .width_range(layout::SIDEBAR_MIN_WIDTH..=layout::SIDEBAR_MAX_WIDTH)
                .resizable(true)
                .frame(sidebar_frame(tokens))
                .show(context, |ui| match snapshot.active_workspace {
                    Workspace::Connections => {
                        connection_launcher::panel(ui, &snapshot, tokens, commands, i18n);
                    }
                    active_workspace => {
                        workspace::sidebar(ui, &snapshot, active_workspace, tokens, commands, i18n);
                    }
                });
        }

        CentralPanel::default()
            .frame(central_frame(tokens))
            .show(context, |ui| {
                workspace::show(ui, &snapshot, tokens, commands, i18n);
            });
        transfer_wizard::show(
            context,
            &snapshot,
            tokens,
            commands,
            i18n,
            &mut self.transfer_wizard,
        );
        toasts::show(context, &snapshot, tokens);
    }

    fn ensure_icons_installed(&mut self, context: &egui::Context) {
        if !self.icons_installed {
            icons::install(context);
            self.icons_installed = true;
        }
    }
}

fn context_panel_visible(snapshot: &AppSnapshot) -> bool {
    !matches!(
        snapshot.active_workspace,
        Workspace::Scripts
            | Workspace::Plugins
            | Workspace::Diagnostics
            | Workspace::Settings
            | Workspace::About
    )
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

fn top_frame(tokens: ThemeTokens) -> Frame {
    Frame::NONE
        .fill(chrome_bg(tokens))
        .inner_margin(egui::Margin::symmetric(
            layout::HEADER_MARGIN_X,
            layout::HEADER_MARGIN_Y,
        ))
}

fn rail_frame(tokens: ThemeTokens) -> Frame {
    Frame::NONE
        .fill(chrome_bg(tokens))
        .inner_margin(egui::Margin::same(layout::RAIL_MARGIN))
}

fn chrome_bg(tokens: ThemeTokens) -> egui::Color32 {
    tokens.panel_raised.gamma_multiply(1.12)
}

fn sidebar_frame(tokens: ThemeTokens) -> Frame {
    Frame::NONE
        .fill(tokens.window_bg)
        .inner_margin(layout::sidebar_margin())
}

fn central_frame(tokens: ThemeTokens) -> Frame {
    Frame::NONE
        .fill(tokens.window_bg)
        .inner_margin(egui::Margin::same(layout::CENTRAL_MARGIN))
}

#[cfg(test)]
mod tests {
    use super::*;
    use correo_style::static_tokens;

    #[test]
    fn header_frame_uses_larger_height_and_horizontal_padding() {
        let frame = top_frame(static_tokens(&ThemeMode::Dark));

        assert_eq!(layout::HEADER_HEIGHT, 64.0);
        assert_eq!(
            frame.inner_margin,
            egui::Margin::symmetric(layout::HEADER_MARGIN_X, layout::HEADER_MARGIN_Y)
        );
    }

    #[test]
    fn global_chrome_frames_do_not_draw_decorative_strokes() {
        let tokens = static_tokens(&ThemeMode::Dark);

        assert_eq!(top_frame(tokens).stroke, egui::Stroke::NONE);
        assert_eq!(rail_frame(tokens).stroke, egui::Stroke::NONE);
        assert_eq!(sidebar_frame(tokens).stroke, egui::Stroke::NONE);
    }
}
