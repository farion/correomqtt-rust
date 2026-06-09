use crate::{
    marketplace_rows_from_repository_json, PluginFeedback, PluginLoadState, PluginSurfaceSnapshot,
    PluginSurfaceTab,
};

const LOCAL_PLUGIN_REPOSITORY_JSON: &str =
    include_str!("../../correo-plugins/tests/fixtures/repository.json");

pub(super) fn plugin_surface(install_bundled_plugins: bool) -> PluginSurfaceSnapshot {
    let Ok(mut marketplace_plugins) =
        marketplace_rows_from_repository_json(LOCAL_PLUGIN_REPOSITORY_JSON)
    else {
        return PluginSurfaceSnapshot {
            load_state: PluginLoadState::Empty,
            feedback: Some(PluginFeedback::error(
                "The bundled plugin repository catalog could not be loaded.",
            )),
            ..PluginSurfaceSnapshot::default()
        };
    };

    let mut plugins = Vec::new();
    if install_bundled_plugins {
        for marketplace_plugin in &mut marketplace_plugins {
            if marketplace_plugin.install_source.is_bundled() {
                marketplace_plugin.installed_plugin_id = Some(marketplace_plugin.id.clone());
                plugins.push(marketplace_plugin.to_installed_plugin());
            }
        }
    }

    let selected_plugin_id = plugins
        .first()
        .map(|plugin| plugin.id.clone())
        .unwrap_or_default();
    let selected_marketplace_plugin_id = marketplace_plugins
        .first()
        .map(|plugin| plugin.id.clone())
        .unwrap_or_default();
    let load_state = if plugins.is_empty() && marketplace_plugins.is_empty() {
        PluginLoadState::Empty
    } else {
        PluginLoadState::Ready
    };

    PluginSurfaceSnapshot {
        active_tab: PluginSurfaceTab::Installed,
        load_state,
        plugins,
        marketplace_plugins,
        selected_plugin_id,
        selected_marketplace_plugin_id,
        ..PluginSurfaceSnapshot::default()
    }
}
