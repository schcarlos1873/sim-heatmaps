//! Commands Tauri: apenas adaptam chamadas da UI para os casos de uso.

use application::dto::{ContourData, GenerateRequest, MachineInput, PropertyInput, Settings};
use application::AppResult;
use domain::machine::Machine;
use domain::property::Property;
use domain::repositories::ReelRecord;
use serde::Serialize;
use tauri::State;

use crate::state::AppState;

type S<'a> = State<'a, AppState>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    version: &'static str,
    db_path: String,
    log_dir: String,
}

#[tauri::command]
pub fn app_info(state: S) -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION"),
        db_path: state.db_path.to_string_lossy().into_owned(),
        log_dir: state.log_dir.to_string_lossy().into_owned(),
    }
}

#[tauri::command]
pub fn list_machines(state: S) -> AppResult<Vec<Machine>> {
    state.machines.list()
}

#[tauri::command]
pub fn create_machine(state: S, input: MachineInput) -> AppResult<Machine> {
    state.machines.create(input)
}

#[tauri::command]
pub fn update_machine(state: S, id: i64, input: MachineInput) -> AppResult<Machine> {
    state.machines.update(id, input)
}

#[tauri::command]
pub fn delete_machine(state: S, id: i64) -> AppResult<()> {
    state.machines.delete(id)
}

#[tauri::command]
pub fn list_properties(state: S, machine_id: i64) -> AppResult<Vec<Property>> {
    state.properties.list(machine_id)
}

#[tauri::command]
pub fn create_property(state: S, input: PropertyInput) -> AppResult<Property> {
    state.properties.create(input)
}

#[tauri::command]
pub fn update_property(state: S, id: i64, input: PropertyInput) -> AppResult<Property> {
    state.properties.update(id, input)
}

#[tauri::command]
pub fn delete_property(state: S, id: i64) -> AppResult<()> {
    state.properties.delete(id)
}

/// Comando assíncrono: a geração grava vários arquivos e não deve bloquear a janela.
#[tauri::command]
pub async fn generate_reel(state: S<'_>, request: GenerateRequest) -> AppResult<ReelRecord> {
    state.generator.generate(request)
}

#[tauri::command]
pub fn list_reels(state: S, machine_id: Option<i64>, limit: Option<u32>) -> AppResult<Vec<ReelRecord>> {
    state.contour.list_reels(machine_id, limit.unwrap_or(500))
}

#[tauri::command]
pub fn delete_reel(state: S, id: i64, delete_files: bool) -> AppResult<()> {
    state.contour.delete_reel(id, delete_files)
}

#[tauri::command]
pub async fn load_contour(state: S<'_>, reel_id: i64, property_code: String) -> AppResult<ContourData> {
    state.contour.load(reel_id, &property_code)
}

#[tauri::command]
pub async fn load_contour_file(state: S<'_>, path: String) -> AppResult<ContourData> {
    state.contour.load_file(&path)
}

#[tauri::command]
pub fn get_settings(state: S) -> AppResult<Settings> {
    state.settings.get()
}

#[tauri::command]
pub fn update_settings(state: S, settings: Settings) -> AppResult<Settings> {
    state.settings.update(settings)
}
