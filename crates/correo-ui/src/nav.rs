use correo_core::{AppCommand, AppCommandSender, Workspace};
use egui::{Align, Button, CornerRadius, Layout, RichText, Stroke, Ui};

use crate::i18n::I18n;
use crate::icons;
use crate::theme::ThemeTokens;

const TOP_WORKSPACES: [Workspace; 2] = [Workspace::Connections, Workspace::Scripts];
const BOTTOM_WORKSPACES: [Workspace; 4] = [
    Workspace::Plugins,
    Workspace::Diagnostics,
    Workspace::Settings,
    Workspace::About,
];
pub fn rail(
    ui: &mut Ui,
    active: Workspace,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        ui.add_space(4.0);
        nav_group(ui, &TOP_WORKSPACES, active, tokens, commands, i18n);
        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            ui.add_space(4.0);
            nav_group_reversed(ui, &BOTTOM_WORKSPACES, active, tokens, commands, i18n);
        });
    });
}

fn nav_group(
    ui: &mut Ui,
    workspaces: &[Workspace],
    active: Workspace,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    for &workspace in workspaces {
        nav_button(ui, workspace, active, tokens, commands, i18n);
        ui.add_space(4.0);
    }
}

fn nav_group_reversed(
    ui: &mut Ui,
    workspaces: &[Workspace],
    active: Workspace,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    for &workspace in workspaces.iter().rev() {
        nav_button(ui, workspace, active, tokens, commands, i18n);
        ui.add_space(4.0);
    }
}

fn nav_button(
    ui: &mut Ui,
    workspace: Workspace,
    active: Workspace,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
    i18n: &I18n,
) {
    let selected = workspace == active;
    let fill = if selected {
        tokens.accent_selected_bg
    } else {
        tokens.panel_bg
    };
    let response = ui
        .add_sized(
            [32.0, 32.0],
            Button::new(RichText::new(icons::workspace_icon(workspace)).size(18.0))
                .fill(fill)
                .stroke(Stroke::new(1.0, tokens.border))
                .corner_radius(CornerRadius::same(4)),
        )
        .on_hover_text(i18n.workspace_label(workspace));
    if selected {
        let rect = response.rect;
        let accent = egui::Rect::from_min_size(rect.left_top(), egui::vec2(3.0, rect.height()));
        ui.painter()
            .rect_filled(accent, CornerRadius::same(1), tokens.accent);
    }
    if response.clicked() {
        let _ = commands.send(AppCommand::SelectWorkspace(workspace));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rail_groups_match_sidebar_spec() {
        assert_eq!(TOP_WORKSPACES, [Workspace::Connections, Workspace::Scripts]);
        assert_eq!(
            BOTTOM_WORKSPACES,
            [
                Workspace::Plugins,
                Workspace::Diagnostics,
                Workspace::Settings,
                Workspace::About,
            ]
        );
    }
}
