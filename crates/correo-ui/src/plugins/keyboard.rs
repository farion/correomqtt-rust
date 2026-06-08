use correo_core::{
    AppCommand, AppCommandSender, PluginLoadState, PluginSurfaceSnapshot, PluginSurfaceTab,
};
use egui::{Context, Id, Key, Modifiers};

const PLUGIN_SEARCH_ID: &str = "plugin-manager-search";
const DIAGNOSTIC_SEARCH_ID: &str = "plugin-diagnostics-search";

pub(super) fn plugin_search_id() -> Id {
    Id::new(PLUGIN_SEARCH_ID)
}

pub(super) fn diagnostic_search_id() -> Id {
    Id::new(DIAGNOSTIC_SEARCH_ID)
}

pub(super) fn handle(
    context: &Context,
    plugins: &PluginSurfaceSnapshot,
    commands: &AppCommandSender,
) {
    if plugins.load_state != PluginLoadState::Ready {
        return;
    }

    if consume_command_key(context, Key::F) {
        let search_id = if plugins.active_tab == PluginSurfaceTab::Diagnostics {
            diagnostic_search_id()
        } else {
            plugin_search_id()
        };
        context.memory_mut(|memory| memory.request_focus(search_id));
        return;
    }

    if consume_command_key(context, Key::S) {
        apply_current_editor(plugins, commands);
        return;
    }

    if context.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape)) {
        cancel_current_editor(plugins, commands);
        return;
    }

    if context.memory(|memory| memory.focused().is_some()) {
        return;
    }

    if context.input_mut(|input| input.consume_key(Modifiers::NONE, Key::ArrowDown)) {
        select_adjacent_plugin(plugins, commands, 1);
        return;
    }
    if context.input_mut(|input| input.consume_key(Modifiers::NONE, Key::ArrowUp)) {
        select_adjacent_plugin(plugins, commands, -1);
        return;
    }
    if context.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Enter)) {
        activate_selected_row(plugins, commands);
        return;
    }
    if context.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Space)) {
        toggle_selected_row(plugins, commands);
    }
}

fn consume_command_key(context: &Context, key: Key) -> bool {
    context.input_mut(|input| input.consume_key(Modifiers::COMMAND, key))
}

fn apply_current_editor(plugins: &PluginSurfaceSnapshot, commands: &AppCommandSender) {
    if plugins.hook_editor.is_some() {
        send(commands, AppCommand::ApplyPluginHookEdit);
        return;
    }

    if plugins.active_tab == PluginSurfaceTab::Configuration {
        if let Some(plugin) = plugins
            .selected_plugin()
            .filter(|plugin| !plugin.config_fields.is_empty())
        {
            send(
                commands,
                AppCommand::ApplyPluginConfig {
                    plugin_id: plugin.id.clone(),
                },
            );
        }
    }
}

fn cancel_current_editor(plugins: &PluginSurfaceSnapshot, commands: &AppCommandSender) {
    if plugins.hook_editor.is_some() {
        send(commands, AppCommand::CancelPluginHookEdit);
    } else if plugins.disable_confirmation.is_some() {
        send(commands, AppCommand::CancelPluginDisable);
    } else if plugins.active_tab == PluginSurfaceTab::Configuration {
        if let Some(plugin) = plugins
            .selected_plugin()
            .filter(|plugin| !plugin.config_fields.is_empty())
        {
            send(
                commands,
                AppCommand::CancelPluginConfig {
                    plugin_id: plugin.id.clone(),
                },
            );
        }
    }
}

fn select_adjacent_plugin(
    plugins: &PluginSurfaceSnapshot,
    commands: &AppCommandSender,
    direction: isize,
) {
    let filtered = plugins.filtered_plugins();
    if filtered.is_empty() {
        return;
    }

    let current = filtered
        .iter()
        .position(|plugin| plugin.id == plugins.selected_plugin_id)
        .unwrap_or(0);
    let last = filtered.len() as isize - 1;
    let next = (current as isize + direction).clamp(0, last) as usize;
    send(
        commands,
        AppCommand::SelectPlugin(filtered[next].id.clone()),
    );
}

fn activate_selected_row(plugins: &PluginSurfaceSnapshot, commands: &AppCommandSender) {
    let Some(plugin) = plugins.selected_plugin() else {
        select_adjacent_plugin(plugins, commands, 0);
        return;
    };

    if plugins.active_tab == PluginSurfaceTab::Hooks {
        if let Some(assignment) = plugin.hooks.first() {
            send(
                commands,
                AppCommand::StartEditPluginHook {
                    plugin_id: plugin.id.clone(),
                    hook: assignment.hook,
                },
            );
        } else {
            send(
                commands,
                AppCommand::StartAddPluginHook {
                    plugin_id: plugin.id.clone(),
                },
            );
        }
    }
}

fn toggle_selected_row(plugins: &PluginSurfaceSnapshot, commands: &AppCommandSender) {
    let Some(plugin) = plugins.selected_plugin() else {
        select_adjacent_plugin(plugins, commands, 0);
        return;
    };

    if plugins.active_tab == PluginSurfaceTab::Hooks {
        if let Some(assignment) = plugin.hooks.first() {
            send(
                commands,
                AppCommand::SetPluginHookEnabled {
                    plugin_id: plugin.id.clone(),
                    hook: assignment.hook,
                    enabled: !assignment.enabled,
                },
            );
        }
    } else {
        send(
            commands,
            AppCommand::SetPluginEnabled {
                plugin_id: plugin.id.clone(),
                enabled: !plugin.enabled,
            },
        );
    }
}

fn send(commands: &AppCommandSender, command: AppCommand) {
    let _ = commands.send(command);
}
