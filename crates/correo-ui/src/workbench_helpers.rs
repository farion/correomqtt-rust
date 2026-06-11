use correo_core::{AppCommand, AppCommandSender, AppSnapshot, ConnectionState, QosLevel};
use egui::{ComboBox, Rect, Ui, UiBuilder};

pub(crate) fn connected(snapshot: &AppSnapshot) -> bool {
    snapshot
        .selected_connection()
        .is_some_and(|connection| connection.state == ConnectionState::Connected)
}

pub(crate) fn qos_selector(
    ui: &mut Ui,
    id: &'static str,
    current: QosLevel,
    mut on_change: impl FnMut(QosLevel),
) {
    let mut selected = current;
    ComboBox::from_id_salt(id)
        .selected_text(current.label())
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            for qos in QosLevel::ALL {
                ui.selectable_value(&mut selected, qos, qos.label());
            }
        });
    if selected != current {
        on_change(selected);
    }
}

pub(crate) fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}

pub(crate) fn toolbar_rect(ui: &mut Ui) -> Rect {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), crate::theme::CONTROL_HEIGHT),
        egui::Sense::hover(),
    );
    rect
}

pub(crate) fn right_rect(row: Rect, width: f32, right_offset: f32) -> Rect {
    Rect::from_min_max(
        egui::pos2(row.right() - right_offset - width, row.top()),
        egui::pos2(row.right() - right_offset, row.bottom()),
    )
}

pub(crate) fn child_ui(ui: &mut Ui, rect: Rect, add: impl FnOnce(&mut Ui)) {
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    child.set_clip_rect(rect.expand(2.0));
    add(&mut child);
}
