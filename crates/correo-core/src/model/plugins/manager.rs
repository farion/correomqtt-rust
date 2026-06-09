use crate::PluginFeedback;

use super::AppModel;

impl AppModel {
    pub(super) fn select_marketplace_plugin(&mut self, plugin_id: String) {
        if self
            .snapshot
            .plugins
            .marketplace_plugins
            .iter()
            .any(|plugin| plugin.id == plugin_id)
        {
            self.snapshot.plugins.selected_marketplace_plugin_id = plugin_id;
            self.snapshot.plugins.feedback = None;
        }
    }

    pub(super) fn install_marketplace_plugin(&mut self, marketplace_plugin_id: String) {
        let Some(index) = self.marketplace_index(&marketplace_plugin_id) else {
            return;
        };
        if let Some(plugin_id) = self.snapshot.plugins.marketplace_plugins[index]
            .installed_plugin_id
            .clone()
        {
            self.snapshot.plugins.selected_plugin_id = plugin_id;
            self.snapshot.plugins.feedback = Some(PluginFeedback::info(
                "Plugin is already installed; selected the installed copy.",
            ));
            return;
        }
        let marketplace_id = self.snapshot.plugins.marketplace_plugins[index].id.clone();
        if self.plugin_index(&marketplace_id).is_some() {
            self.snapshot.plugins.marketplace_plugins[index].installed_plugin_id =
                Some(marketplace_id.clone());
            self.snapshot.plugins.selected_plugin_id = marketplace_id;
            self.snapshot.plugins.feedback = Some(PluginFeedback::info(
                "Plugin is already installed; linked the marketplace entry.",
            ));
            return;
        }

        let plugin = self.snapshot.plugins.marketplace_plugins[index].to_installed_plugin();
        let plugin_id = plugin.id.clone();
        let plugin_name = plugin.name.clone();
        self.snapshot.plugins.plugins.push(plugin);
        self.snapshot.plugins.marketplace_plugins[index].installed_plugin_id =
            Some(plugin_id.clone());
        self.snapshot.plugins.selected_plugin_id = plugin_id;
        self.snapshot.plugins.feedback =
            Some(PluginFeedback::info(format!("{plugin_name} installed.")));
    }

    pub(super) fn uninstall_plugin(&mut self, plugin_id: String) {
        let Some(index) = self.plugin_index(&plugin_id) else {
            return;
        };
        let plugin = self.snapshot.plugins.plugins.remove(index);
        for marketplace_plugin in &mut self.snapshot.plugins.marketplace_plugins {
            if marketplace_plugin.installed_plugin_id.as_deref() == Some(&plugin.id) {
                marketplace_plugin.installed_plugin_id = None;
            }
        }
        if self
            .snapshot
            .plugins
            .disable_confirmation
            .as_ref()
            .is_some_and(|confirmation| confirmation.plugin_id == plugin.id)
        {
            self.snapshot.plugins.disable_confirmation = None;
        }
        if self.snapshot.plugins.selected_plugin_id == plugin.id {
            self.snapshot.plugins.selected_plugin_id = self
                .snapshot
                .plugins
                .plugins
                .first()
                .map(|plugin| plugin.id.clone())
                .unwrap_or_default();
        }
        self.snapshot.plugins.feedback = Some(PluginFeedback::warning(format!(
            "{} uninstalled.",
            plugin.name
        )));
    }

    fn marketplace_index(&self, plugin_id: &str) -> Option<usize> {
        self.snapshot
            .plugins
            .marketplace_plugins
            .iter()
            .position(|plugin| plugin.id == plugin_id)
    }
}
