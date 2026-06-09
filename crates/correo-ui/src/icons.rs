use correo_core::Workspace;
use egui_phosphor::regular;

pub(crate) fn install(context: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    context.set_fonts(fonts);
}

pub(crate) fn workspace_icon(workspace: Workspace) -> &'static str {
    match workspace {
        Workspace::Connections => regular::LIST_BULLETS,
        Workspace::ImportExport => regular::TROLLEY_SUITCASE,
        Workspace::Scripts => regular::SCROLL,
        Workspace::Plugins => regular::PACKAGE,
        Workspace::Diagnostics => regular::BUG,
        Workspace::Settings => regular::GEAR,
        Workspace::About => regular::INFO,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_icons_match_sidebar_spec() {
        assert_eq!(
            workspace_icon(Workspace::Connections),
            regular::LIST_BULLETS
        );
        assert_eq!(
            workspace_icon(Workspace::ImportExport),
            regular::TROLLEY_SUITCASE
        );
        assert_eq!(workspace_icon(Workspace::Scripts), regular::SCROLL);
        assert_eq!(workspace_icon(Workspace::Plugins), regular::PACKAGE);
        assert_eq!(workspace_icon(Workspace::Diagnostics), regular::BUG);
        assert_eq!(workspace_icon(Workspace::Settings), regular::GEAR);
        assert_eq!(workspace_icon(Workspace::About), regular::INFO);
    }
}
