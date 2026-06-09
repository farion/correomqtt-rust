use correo_core::{AppCommandSender, AppSnapshot};
use egui::{Context, Id, Order, Window};

use crate::{theme::ThemeTokens, workbench_detail};

pub(crate) fn open_incoming_message(ctx: &Context, message_id: u32) {
    let mut ids = incoming_ids(ctx);
    if !ids.contains(&message_id) {
        ids.push(message_id);
    }
    ctx.data_mut(|data| {
        data.insert_temp(incoming_ids_id(), ids);
        data.insert_temp(focused_incoming_id(), Some(message_id));
    });
}

pub(crate) fn open_outgoing_message(ctx: &Context, row_index: usize) {
    let mut ids = outgoing_ids(ctx);
    if !ids.contains(&row_index) {
        ids.push(row_index);
    }
    ctx.data_mut(|data| {
        data.insert_temp(outgoing_ids_id(), ids);
        data.insert_temp(focused_outgoing_id(), Some(row_index));
    });
}

pub(crate) fn show(
    ctx: &Context,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    show_outgoing(ctx, snapshot, tokens);
    show_incoming(ctx, snapshot, tokens, commands);
}

fn show_incoming(
    ctx: &Context,
    snapshot: &AppSnapshot,
    tokens: ThemeTokens,
    commands: &AppCommandSender,
) {
    let focused = ctx.data_mut(|data| data.get_temp::<Option<u32>>(focused_incoming_id()));
    let mut retained = Vec::new();
    for message_id in incoming_ids(ctx) {
        let Some(message) = snapshot
            .workbench
            .messages
            .iter()
            .find(|message| message.id == message_id)
        else {
            continue;
        };
        let mut open = true;
        Window::new(format!("Message {}", message.topic))
            .id(Id::new(("workbench-message-window", message_id)))
            .order(order_for(focused.flatten() == Some(message_id)))
            .open(&mut open)
            .default_width(520.0)
            .default_height(360.0)
            .show(ctx, |ui| {
                workbench_detail::message_window_content(ui, snapshot, message, tokens, commands);
            });
        if open {
            retained.push(message_id);
        }
    }
    ctx.data_mut(|data| data.insert_temp(incoming_ids_id(), retained));
}

fn show_outgoing(ctx: &Context, snapshot: &AppSnapshot, tokens: ThemeTokens) {
    let focused = ctx.data_mut(|data| data.get_temp::<Option<usize>>(focused_outgoing_id()));
    let mut retained = Vec::new();
    for row_index in outgoing_ids(ctx) {
        let Some(row) = snapshot.workbench.publish.history.get(row_index) else {
            continue;
        };
        let mut open = true;
        Window::new(format!("Published {}", row.topic))
            .id(Id::new(("workbench-outgoing-message-window", row_index)))
            .order(order_for(focused.flatten() == Some(row_index)))
            .open(&mut open)
            .default_width(480.0)
            .default_height(260.0)
            .show(ctx, |ui| {
                workbench_detail::outgoing_window_content(ui, row, tokens);
            });
        if open {
            retained.push(row_index);
        }
    }
    ctx.data_mut(|data| data.insert_temp(outgoing_ids_id(), retained));
}

fn order_for(focused: bool) -> Order {
    if focused {
        Order::Foreground
    } else {
        Order::Middle
    }
}

fn incoming_ids(ctx: &Context) -> Vec<u32> {
    ctx.data_mut(|data| data.get_temp(incoming_ids_id()).unwrap_or_default())
}

fn outgoing_ids(ctx: &Context) -> Vec<usize> {
    ctx.data_mut(|data| data.get_temp(outgoing_ids_id()).unwrap_or_default())
}

fn incoming_ids_id() -> Id {
    Id::new("workbench-open-incoming-message-windows")
}

fn outgoing_ids_id() -> Id {
    Id::new("workbench-open-outgoing-message-windows")
}

fn focused_incoming_id() -> Id {
    Id::new("workbench-focused-incoming-message-window")
}

fn focused_outgoing_id() -> Id {
    Id::new("workbench-focused-outgoing-message-window")
}
