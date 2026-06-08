use correo_core::{
    AppRuntime, Diagnostic, HistoryPersistenceWorker, MqttService, RumqttSessionFactory,
    SettingsPersistenceWorker,
};

use crate::startup::{history_root, load_startup_state};

pub fn run() -> eframe::Result {
    correo_diagnostics::install_tracing();
    tracing::info!("starting CorreoMQTT desktop shell");

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_app_id("org.correomqtt.CorreoMQTT")
            .with_title("CorreoMQTT")
            .with_icon(app_icon())
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([900.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "CorreoMQTT",
        options,
        Box::new(|creation_context| Ok(Box::new(CorreoDesktopApp::new(creation_context)))),
    )
}

fn app_icon() -> eframe::egui::IconData {
    eframe::icon_data::from_png_bytes(include_bytes!("../../../icon/ico/Icon_256x256.png"))
        .unwrap_or_default()
}

struct CorreoDesktopApp {
    runtime: AppRuntime,
    _mqtt_runtime: Option<tokio::runtime::Runtime>,
    ui: correo_ui::CorreoUi,
}

impl CorreoDesktopApp {
    fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        let theme_mode = correo_ui::stored_theme(creation_context);
        let mut runtime = AppRuntime::with_startup_state(load_startup_state(theme_mode));
        let mqtt_runtime = attach_mqtt_service(&mut runtime);
        runtime.attach_history_worker(HistoryPersistenceWorker::start(history_root()));
        runtime.attach_settings_worker(SettingsPersistenceWorker::start(history_root()));
        let ui = correo_ui::CorreoUi::with_command_sender(
            creation_context,
            runtime.snapshot().clone(),
            runtime.command_sender(),
        );
        Self {
            runtime,
            _mqtt_runtime: mqtt_runtime,
            ui,
        }
    }

    fn pump_runtime(&mut self, context: &eframe::egui::Context) {
        let report = self.runtime.pump();
        if report.snapshot_changed {
            self.ui.set_snapshot(self.runtime.snapshot().clone());
            context.request_repaint();
        }
        if report.shutdown_requested {
            context.send_viewport_cmd(eframe::egui::ViewportCommand::Close);
        }
    }
}

fn attach_mqtt_service(runtime: &mut AppRuntime) -> Option<tokio::runtime::Runtime> {
    let mqtt_runtime = match tokio::runtime::Builder::new_multi_thread()
        .thread_name("correo-mqtt")
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            record_startup_diagnostic(
                runtime,
                format!("MQTT runtime could not be started: {error}"),
            );
            return None;
        }
    };

    let service = {
        let _guard = mqtt_runtime.enter();
        MqttService::spawn(RumqttSessionFactory)
    };
    match service {
        Ok(service) => runtime.attach_mqtt_service(service),
        Err(error) => {
            record_startup_diagnostic(
                runtime,
                format!("MQTT service could not be started: {error}"),
            );
        }
    }
    Some(mqtt_runtime)
}

fn record_startup_diagnostic(runtime: &mut AppRuntime, message: String) {
    let _ = runtime
        .event_sender()
        .emit(correo_core::AppEvent::DiagnosticRaised(Diagnostic::error(
            message,
        )));
    runtime.pump();
}

impl eframe::App for CorreoDesktopApp {
    fn update(&mut self, context: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.pump_runtime(context);
        self.ui.draw(context);
        self.pump_runtime(context);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(
            storage,
            correo_ui::THEME_KEY,
            &self.runtime.snapshot().theme_mode,
        );
    }
}
