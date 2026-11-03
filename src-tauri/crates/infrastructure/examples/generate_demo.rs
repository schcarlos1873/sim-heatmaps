//! Gera uma bobina de demonstração da máquina PM1 em uma pasta (use uma pasta fora do projeto).
//!
//! `cargo run -p infrastructure --example generate_demo -- <pasta> [scans] [seed]`

use std::sync::Arc;

use application::dto::{GenerateRequest, Settings};
use application::services::{GeneratorService, MachineService, SettingsService};
use domain::generator::GeneratorParams;
use infrastructure::repositories::*;
use infrastructure::scan_csv::ScanCsvStorage;
use infrastructure::Database;

fn main() {
    let mut args = std::env::args().skip(1);
    let dir = args.next().expect("uso: generate_demo <pasta> [scans] [seed]");
    let scans: u32 = args.next().map(|s| s.parse().expect("scans")).unwrap_or(120);
    let seed: Option<u64> = args.next().map(|s| s.parse().expect("seed"));

    let db = Database::open_in_memory().expect("db");
    let m = Arc::new(SqliteMachineRepository(db.clone()));
    let p = Arc::new(SqlitePropertyRepository(db.clone()));
    let settings = Arc::new(SettingsService::new(Arc::new(SqliteSettingsRepository(db.clone())), dir.clone()));
    settings.update(Settings { export_dir: dir, contour_colors: Default::default() }).expect("settings");
    let machines = MachineService::new(m.clone(), p.clone());
    machines.seed_if_empty().expect("seed");
    let pm1 = machines.list().expect("machines").remove(0);
    let generator = GeneratorService::new(m, p, Arc::new(SqliteReelRepository(db)), Arc::new(ScanCsvStorage), settings);

    let reel = generator
        .generate(GenerateRequest {
            machine_id: pm1.id.unwrap(),
            reel_id: "PM1-DEMO".into(),
            started_at: "2026-11-02T08:00".into(),
            scan_count: Some(scans),
            duration_min: None,
            property_ids: None,
            params: GeneratorParams::default(),
            seed,
            overwrite: true,
        })
        .unwrap_or_else(|e| panic!("{}: {}", e.code, e.message));
    for f in &reel.files {
        println!("{}", f.file_path);
    }
}
