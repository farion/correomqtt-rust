use crate::{
    PluginConfigField, PluginHookAssignment, PluginHookDraft, PluginHookStatus, PluginRow,
    PluginStatus,
};

pub(super) fn disable_plugin(plugin: &mut PluginRow) {
    plugin.enabled = false;
    plugin.status = PluginStatus::Disabled;
    for hook in &mut plugin.hooks {
        hook.enabled = false;
        hook.status = PluginHookStatus::Disabled;
    }
}

pub(super) fn set_field_validation(field: &mut PluginConfigField) {
    field.error = validate_field(field).err();
    field.valid = field.error.is_none();
}

pub(super) fn validate_config_fields(fields: &mut [PluginConfigField]) -> Option<String> {
    let mut first_error = None;
    for field in fields {
        set_field_validation(field);
        if first_error.is_none() {
            first_error = field.error.clone();
        }
    }
    first_error
}

pub(super) fn refresh_plugin_config_status(plugin: &mut PluginRow) {
    if plugin.config_fields.iter().any(|field| !field.valid) {
        plugin.status = PluginStatus::NeedsConfig;
    } else if plugin.status == PluginStatus::NeedsConfig {
        plugin.status = if plugin.enabled {
            PluginStatus::Active
        } else {
            PluginStatus::Disabled
        };
    }
}

pub(super) fn restore_config_values(plugin: &mut PluginRow) {
    for field in &mut plugin.config_fields {
        if !field.sensitive {
            field.value = field.saved_value.clone();
        }
        field.error = None;
        field.valid = true;
    }
    refresh_plugin_config_status(plugin);
}

pub(super) fn validate_hook_draft(draft: &PluginHookDraft) -> Result<(), String> {
    if draft.target.trim().is_empty() {
        return Err("Target topic/filter is required.".to_owned());
    }
    parse_json_object(&draft.config_json).map_err(|error| format!("Config JSON {error}"))?;
    Ok(())
}

pub(super) fn assignment_from_draft(draft: &PluginHookDraft) -> PluginHookAssignment {
    PluginHookAssignment {
        hook: draft.hook,
        enabled: draft.enabled,
        target: draft.target.clone(),
        config_json: draft.config_json.clone(),
        status: if draft.enabled {
            PluginHookStatus::Ready
        } else {
            PluginHookStatus::Disabled
        },
        last_run: "never".to_owned(),
        message: "Assignment saved".to_owned(),
    }
}

fn validate_field(field: &PluginConfigField) -> Result<(), String> {
    if field.sensitive {
        return Ok(());
    }
    let value = field.value.trim();
    if field.required && value.is_empty() {
        return Err(format!("{} is required.", field.label));
    }

    let schema = field.schema_hint.trim();
    if schema.eq_ignore_ascii_case("integer") {
        value
            .parse::<i64>()
            .map(|_| ())
            .map_err(|_| format!("{} must be an integer.", field.label))
    } else if schema.eq_ignore_ascii_case("boolean") {
        value
            .parse::<bool>()
            .map(|_| ())
            .map_err(|_| format!("{} must be true or false.", field.label))
    } else if schema.eq_ignore_ascii_case("JSON object") {
        parse_json_object(value).map_err(|error| format!("{} {error}", field.label))
    } else if schema.contains('|') {
        let choices: Vec<_> = schema.split('|').map(str::trim).collect();
        if choices.contains(&value) {
            Ok(())
        } else {
            Err(format!(
                "{} must be one of: {}.",
                field.label,
                choices.join(", ")
            ))
        }
    } else {
        Ok(())
    }
}

fn parse_json_object(value: &str) -> Result<(), String> {
    let parsed = serde_json::from_str::<serde_json::Value>(value)
        .map_err(|error| format!("must be valid JSON: {error}."))?;
    if parsed.is_object() {
        Ok(())
    } else {
        Err("must be a JSON object.".to_owned())
    }
}
