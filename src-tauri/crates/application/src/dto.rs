use chrono::NaiveDateTime;
use domain::colors::ContourColors;
use domain::generator::GeneratorParams;
use domain::machine::{Machine, ScannerModel};
use domain::property::{Limits, Property};
use domain::scan::{ScanDirection, SensorStatus};
use domain::stats::{CdProfiles, Summary};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MachineInput {
    pub name: String,
    pub scanner_model: ScannerModel,
    pub width_mm: f64,
    /// Vazio = 800 bins.
    pub bin_count: Option<u32>,
    pub speed_m_min: f64,
    pub scan_time_s: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyInput {
    pub machine_id: i64,
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub unit: String,
    pub decimals: u8,
    pub sort_order: Option<i32>,
    #[serde(default = "yes")]
    pub active: bool,
    pub limits: Limits,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateRequest {
    pub machine_id: i64,
    pub reel_id: String,
    /// `YYYY-MM-DDTHH:MM` ou `YYYY-MM-DDTHH:MM:SS` (input datetime-local).
    pub started_at: String,
    /// Quantidade de scans; se vazio, calculada a partir de `duration_min`.
    pub scan_count: Option<u32>,
    pub duration_min: Option<f64>,
    /// Vazio = todas as propriedades ativas da máquina.
    pub property_ids: Option<Vec<i64>>,
    #[serde(default)]
    pub params: GeneratorParams,
    /// Semente para reproduzir a mesma bobina.
    pub seed: Option<u64>,
    #[serde(default)]
    pub overwrite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub export_dir: String,
    /// Cores do Contour Map por limite (configuráveis; ausente = padrão).
    #[serde(default)]
    pub contour_colors: ContourColors,
}

/// Metadados de um scan exibidos no rodapé e nos marcadores do Contour Map.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanInfo {
    pub number: u32,
    pub direction: ScanDirection,
    pub started_at: NaiveDateTime,
    pub ended_at: NaiveDateTime,
    pub speed: f64,
    pub md_position_m: f64,
    pub edge_front_mm: f64,
    pub edge_back_mm: f64,
    pub sheet_break: bool,
    pub sensor_status: SensorStatus,
    pub sensor_temp_c: f64,
}

/// Dados do Contour Map de uma propriedade: matriz (linhas = scans, colunas = bins) e agregados.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContourData {
    pub reel_id: String,
    pub file_path: String,
    pub machine: Machine,
    pub property: Property,
    pub bin_centers: Vec<f64>,
    pub scans: Vec<ScanInfo>,
    pub values: Vec<Vec<Option<f64>>>,
    pub cd: CdProfiles,
    pub md: Vec<Option<f64>>,
    pub summary: Option<Summary>,
}
