use std::time::Duration;

use correo_core::{AppSnapshot, Diagnostic, DiagnosticSeverity};
use correo_style::ThemeTokens;
use egui::{Align2, Area, Frame, Id, RichText};
use time::OffsetDateTime;

const SCRIPT_TOAST_MESSAGES: [&str; 3] = [
    "Script execution started.",
    "Script execution succeeded.",
    "Script execution failed.",
];

pub(crate) fn show(context: &egui::Context, snapshot: &AppSnapshot, tokens: ThemeTokens) {
    let toasts = snapshot
        .diagnostics
        .iter()
        .filter(|diagnostic| is_toast_diagnostic(diagnostic))
        .filter(|diagnostic| is_recent(diagnostic))
        .take(3)
        .collect::<Vec<_>>();
    if toasts.is_empty() {
        return;
    }

    context.request_repaint_after(Duration::from_millis(250));
    Area::new(Id::new("script-event-toasts"))
        .order(egui::Order::Tooltip)
        .anchor(Align2::RIGHT_BOTTOM, egui::vec2(-16.0, -16.0))
        .show(context, |ui| {
            ui.set_width(300.0);
            ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                for diagnostic in toasts {
                    let background = severity_color(diagnostic.severity, tokens);
                    Frame::NONE
                        .fill(background)
                        .stroke(egui::Stroke::NONE)
                        .corner_radius(egui::CornerRadius::same(6))
                        .inner_margin(egui::Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(&diagnostic.message)
                                    .strong()
                                    .color(contrast_text(background)),
                            );
                        });
                    ui.add_space(6.0);
                }
            });
        });
}

fn is_recent(diagnostic: &Diagnostic) -> bool {
    let age = OffsetDateTime::now_utc() - diagnostic.occurred_at;
    !age.is_negative() && age <= time::Duration::seconds(5)
}

fn is_toast_diagnostic(diagnostic: &Diagnostic) -> bool {
    SCRIPT_TOAST_MESSAGES.contains(&diagnostic.message.as_str())
        || is_script_feedback_message(&diagnostic.message)
        || is_connection_settings_feedback_message(&diagnostic.message)
}

fn is_connection_settings_feedback_message(message: &str) -> bool {
    matches!(
        message,
        "Name is required"
            | "Host is required"
            | "SSL keystore is required when TLS/SSL uses Keystore"
            | "SSH host is required"
            | "SSH username is required"
            | "SSH key file is required"
    ) || message.ends_with(" is required")
        || message.ends_with(" must be between 1 and 65535")
}

fn is_script_feedback_message(message: &str) -> bool {
    matches!(
        message,
        "Select an available connection."
            | "Script name is required."
            | "Use forward slashes for script folders."
            | "Script names cannot use sidecar storage folders."
            | "Script name must be a safe relative .js path."
            | "Select a script before saving."
            | "Select a script before discarding changes."
            | "Select a script before renaming."
            | "Select a script before deleting."
            | "Cancel the running script before deleting it."
            | "A script execution is already running."
            | "Select a script before running."
            | "No running script to cancel."
            | "No finished execution logs to clear."
    ) || message.starts_with("Created ")
        || message.starts_with("Saved ")
        || message.starts_with("Discarded changes to ")
        || message.starts_with("Renamed ")
        || message.starts_with("Deleted ")
        || message.starts_with("Script already exists: ")
        || (message.starts_with("Cleared ") && message.ends_with(" finished execution log(s)."))
}

fn severity_color(severity: DiagnosticSeverity, tokens: ThemeTokens) -> egui::Color32 {
    match severity {
        DiagnosticSeverity::Info => tokens.success,
        DiagnosticSeverity::Warning => tokens.warning,
        DiagnosticSeverity::Error => tokens.danger,
    }
}

fn contrast_text(background: egui::Color32) -> egui::Color32 {
    let [red, green, blue, _] = background.to_array();
    let luminance = 0.299 * red as f32 + 0.587 * green as f32 + 0.114 * blue as f32;
    if luminance > 140.0 {
        egui::Color32::from_rgb(0x17, 0x20, 0x2A)
    } else {
        egui::Color32::WHITE
    }
}
