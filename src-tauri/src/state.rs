use std::path::{Path, PathBuf};
use std::sync::Arc;

use application::services::{ContourService, GeneratorService, MachineService, PropertyService, SettingsService};
use application::AppResult;
use infrastructure::repositories::{
    SqliteMachineRepository, SqlitePropertyRepository, SqliteReelRepository, SqliteSettingsRepository,
};
use infrastructure::scan_csv::ScanCsvStorage;
use infrastructure::Database;

/// Composition root: liga as implementações de infraestrutura aos casos de uso.
pub struct AppState {
    pub machines: MachineService,
    pub properties: PropertyService,
    pub generator: GeneratorService,
    pub contour: ContourService,
    pub settings: Arc<SettingsService>,
    pub db_path: PathBuf,
    pub log_dir: PathBuf,
}

impl AppState {
    pub fn build(app_data: &Path, log_dir: PathBuf, default_export: PathBuf) -> AppResult<Self> {
        let db_path = app_data.join("sim-heatmaps.db");
        let db = Database::open(&db_path)?;
        let machines = Arc::new(SqliteMachineRepository(db.clone()));
        let properties = Arc::new(SqlitePropertyRepository(db.clone()));
        let reels = Arc::new(SqliteReelRepository(db.clone()));
        let storage = Arc::new(ScanCsvStorage);
        let settings = Arc::new(SettingsService::new(
            Arc::new(SqliteSettingsRepository(db)),
            default_export.to_string_lossy().into_owned(),
        ));
        let machine_service = MachineService::new(machines.clone(), properties.clone());
        if machine_service.seed_if_empty()? {
            tracing::info!("database seeded with machine PM1");
        }
        Ok(Self {
            machines: machine_service,
            properties: PropertyService::new(machines.clone(), properties.clone()),
            generator: GeneratorService::new(machines, properties, reels.clone(), storage.clone(), settings.clone()),
            contour: ContourService::new(reels, storage),
            settings,
            db_path,
            log_dir,
        })
    }
}
