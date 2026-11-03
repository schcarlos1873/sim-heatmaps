//! Gera bobinas de demonstração e imprime em JSON as respostas dos commands usadas pela UI.
//! Usado por `tmp/capture_screens.py` para fotografar a interface com dados reais do backend.
//!
//! `cargo run -p infrastructure --example mock_data -- <pasta> [scans] [seed]`

use std::collections::BTreeMap;
use std::sync::Arc;

use application::dto::{GenerateRequest, Settings};
use application::services::{ContourService, GeneratorService, MachineService, PropertyService, SettingsService};
use domain::generator::GeneratorParams;
use infrastructure::repositories::*;
use infrastructure::scan_csv::ScanCsvStorage;
use infrastructure::Database;
use serde_json::json;

fn main() {
    let mut args = std::env::args().skip(1);
    let dir = args.next().expect("uso: mock_data <pasta> [scans] [seed]");
    let scans: u32 = args.next().map(|s| s.parse().expect("scans")).unwrap_or(240);
    let seed: u64 = args.next().map(|s| s.parse().expect("seed")).unwrap_or(2026);

    let db = Database::open_in_memory().expect("db");
    let m = Arc::new(SqliteMachineRepository(db.clone()));
    let p = Arc::new(SqlitePropertyRepository(db.clone()));
    let r = Arc::new(SqliteReelRepository(db.clone()));
    let storage = Arc::new(ScanCsvStorage);
    let settings = Arc::new(SettingsService::new(Arc::new(SqliteSettingsRepository(db)), dir.clone()));
    settings.update(Settings { export_dir: dir, contour_colors: Default::default() }).expect("settings");
    let machines = MachineService::new(m.clone(), p.clone());
    machines.seed_if_empty().expect("seed");
    let properties = PropertyService::new(m.clone(), p.clone());
    let generator = GeneratorService::new(m, p, r.clone(), storage.clone(), settings.clone());
    let contour = ContourService::new(r, storage);

    let pm1 = machines.list().expect("machines").remove(0);
    let request = |reel_id: &str, started_at: &str, scan_count: u32, seed: u64| GenerateRequest {
        machine_id: pm1.id.unwrap(),
        reel_id: reel_id.into(),
        started_at: started_at.into(),
        scan_count: Some(scan_count),
        duration_min: None,
        property_ids: None,
        params: GeneratorParams { sheet_break_probability: 0.006, sensor_fault_probability: 0.01, ..Default::default() },
        seed: Some(seed),
        overwrite: true,
    };
    let fail = |e: application::AppError| -> ! { panic!("{}: {}", e.code, e.message) };
    generator.generate(request("PM1-20261101-2200", "2026-11-01T22:00", scans / 2, seed + 1)).unwrap_or_else(|e| fail(e));
    let main_reel = generator.generate(request("PM1-20261102-0800", "2026-11-02T08:00", scans, seed)).unwrap_or_else(|e| fail(e));

    let reels = contour.list_reels(None, 50).unwrap_or_else(|e| fail(e));
    let mut contours = BTreeMap::new();
    for f in &main_reel.files {
        contours.insert(f.property_code.clone(), contour.load(main_reel.id.unwrap(), &f.property_code).unwrap_or_else(|e| fail(e)));
    }
    let out = json!({
        "machines": machines.list().unwrap_or_else(|e| fail(e)),
        "properties": properties.list(pm1.id.unwrap()).unwrap_or_else(|e| fail(e)),
        "reels": reels,
        "mainReelId": main_reel.id,
        "contours": contours,
        "settings": settings.get().unwrap_or_else(|e| fail(e)),
    });
    println!("{out}");
}
