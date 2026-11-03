//! Camada de apresentação (Tauri): composição das dependências e commands expostos à UI.

mod commands;
mod state;

use std::path::PathBuf;

use tauri::Manager;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Pasta de logs: `<projeto>/log` em desenvolvimento, `<app data>/log` em produção.
fn log_dir(app_data: &std::path::Path) -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("log")
    } else {
        app_data.join("log")
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data = app.path().app_data_dir()?;
            let logs = log_dir(&app_data);
            std::fs::create_dir_all(&logs)?;
            let (writer, guard) = tracing_appender::non_blocking(tracing_appender::rolling::daily(&logs, "sim-heatmaps.log"));
            tracing_subscriber::registry()
                .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
                .with(fmt::layer().with_writer(writer).with_ansi(false))
                .with(fmt::layer().with_writer(std::io::stderr))
                .try_init()
                .ok();

            let default_export = app
                .path()
                .document_dir()
                .map(|d| d.join("sim-heatmaps").join("export"))
                .unwrap_or_else(|_| app_data.join("export"));
            let state = state::AppState::build(&app_data, logs, default_export).map_err(|e| {
                tracing::error!("startup failed: {}", e.message);
                e.message
            })?;
            app.manage(state);
            app.manage(guard);
            tracing::info!(data = ?app_data, "sim-heatmaps started");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::list_machines,
            commands::create_machine,
            commands::update_machine,
            commands::delete_machine,
            commands::list_properties,
            commands::create_property,
            commands::update_property,
            commands::delete_property,
            commands::generate_reel,
            commands::list_reels,
            commands::delete_reel,
            commands::load_contour,
            commands::load_contour_file,
            commands::get_settings,
            commands::update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running sim-heatmaps");
}
