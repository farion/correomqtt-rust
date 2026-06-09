use super::AppModel;
use crate::{AppCommand, ConnectionSurface};

#[test]
fn connection_selection_opens_workbench_and_preserves_pending_editor() {
    let mut model = AppModel::default();
    let first_id = model.snapshot().connections[0].id;
    let second_id = model.snapshot().connections[1].id;

    model.apply_command(AppCommand::OpenConnectionSettings(first_id));

    assert_eq!(model.snapshot().selected_connection, Some(first_id));
    assert_eq!(
        model.snapshot().connection_surface,
        ConnectionSurface::Workbench
    );
    assert_eq!(model.snapshot().connection_settings_overlay, Some(first_id));

    model.apply_command(AppCommand::SelectConnection(second_id));

    assert_eq!(model.snapshot().selected_connection, Some(second_id));
    assert_eq!(
        model.snapshot().connection_surface,
        ConnectionSurface::Workbench
    );
    assert_eq!(model.snapshot().connection_settings_overlay, Some(first_id));

    model.apply_command(AppCommand::SelectConnection(first_id));
    assert_eq!(model.snapshot().connection_settings_overlay, Some(first_id));

    model.apply_command(AppCommand::DiscardConnectionSettings);
    assert_eq!(model.snapshot().connection_settings_overlay, None);
}

#[test]
fn launcher_command_selects_first_connection_workbench() {
    let mut model = AppModel::default();
    let first_id = model.snapshot().connections[0].id;

    model.apply_command(AppCommand::AddConnection);
    assert_eq!(model.snapshot().selected_connection, None);

    model.apply_command(AppCommand::OpenConnectionLauncher);

    assert_eq!(model.snapshot().selected_connection, Some(first_id));
    assert_eq!(
        model.snapshot().connection_surface,
        ConnectionSurface::Workbench
    );
}

#[test]
fn move_connection_reorders_visible_connections() {
    let mut model = AppModel::default();
    let original: Vec<_> = model
        .snapshot()
        .connections
        .iter()
        .map(|connection| connection.id)
        .collect();

    model.apply_command(AppCommand::MoveConnection {
        connection_id: original[0],
        target_connection_id: original[2],
        after: true,
    });

    let reordered: Vec<_> = model
        .snapshot()
        .connections
        .iter()
        .map(|connection| connection.id)
        .collect();
    assert_eq!(
        reordered,
        [original[1], original[2], original[0], original[3]]
    );

    model.apply_command(AppCommand::MoveConnection {
        connection_id: original[0],
        target_connection_id: original[1],
        after: false,
    });

    let restored_front: Vec<_> = model
        .snapshot()
        .connections
        .iter()
        .map(|connection| connection.id)
        .collect();
    assert_eq!(restored_front, original);
}
