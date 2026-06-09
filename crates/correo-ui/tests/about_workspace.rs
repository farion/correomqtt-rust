use correo_core::{sample_snapshot, ThemeMode, Workspace};
use correo_ui::CorreoUi;
use egui_kittest::Harness;

#[test]
fn about_workspace_renders_from_rail_route() {
    assert!(Workspace::ALL.contains(&Workspace::About));

    let mut snapshot = sample_snapshot(ThemeMode::Light);
    snapshot.active_workspace = Workspace::About;
    let mut shell = CorreoUi::for_snapshot(snapshot);

    let mut harness = Harness::builder()
        .with_size(egui::vec2(640.0, 480.0))
        .build(move |context| {
            shell.draw(context);
        });

    harness.run();
}
