//! Testes de integração: serviços de aplicação sobre SQLite em memória e arquivos em pasta temporária.

use std::path::Path;
use std::sync::Arc;

use application::dto::{GenerateRequest, MachineInput, PropertyInput, Settings};
use application::services::{ContourService, GeneratorService, MachineService, PropertyService, SettingsService};
use domain::colors::ContourColors;
use domain::generator::GeneratorParams;
use domain::machine::ScannerModel;
use domain::property::Limits;
use infrastructure::repositories::*;
use infrastructure::scan_csv::ScanCsvStorage;
use infrastructure::Database;

struct Ctx {
    machines: MachineService,
    properties: PropertyService,
    generator: GeneratorService,
    contour: ContourService,
    dir: tempfile::TempDir,
}

fn ctx() -> Ctx {
    let db = Database::open_in_memory().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let m = Arc::new(SqliteMachineRepository(db.clone()));
    let p = Arc::new(SqlitePropertyRepository(db.clone()));
    let r = Arc::new(SqliteReelRepository(db.clone()));
    let storage = Arc::new(ScanCsvStorage);
    let settings = Arc::new(SettingsService::new(Arc::new(SqliteSettingsRepository(db)), "unused".into()));
    settings.update(Settings { export_dir: dir.path().to_string_lossy().into_owned(), contour_colors: Default::default() }).unwrap();
    let machines = MachineService::new(m.clone(), p.clone());
    machines.seed_if_empty().unwrap();
    Ctx {
        machines,
        properties: PropertyService::new(m.clone(), p.clone()),
        generator: GeneratorService::new(m, p, r.clone(), storage.clone(), settings),
        contour: ContourService::new(r, storage),
        dir,
    }
}

fn request(machine_id: i64, reel_id: &str) -> GenerateRequest {
    GenerateRequest {
        machine_id,
        reel_id: reel_id.into(),
        started_at: "2026-11-02T08:00".into(),
        scan_count: Some(60),
        duration_min: None,
        property_ids: None,
        params: GeneratorParams::default(),
        seed: Some(42),
        overwrite: false,
    }
}

fn pm1(c: &Ctx) -> i64 {
    c.machines.list().unwrap().into_iter().find(|m| m.name == "PM1").unwrap().id.unwrap()
}

#[test]
fn seed_creates_pm1_with_default_properties() {
    let c = ctx();
    assert!(!c.machines.seed_if_empty().unwrap());
    let machines = c.machines.list().unwrap();
    assert_eq!(machines.len(), 1);
    assert_eq!((machines[0].bin_count, machines[0].width_mm), (800, 8000.0));
    let props = c.properties.list(pm1(&c)).unwrap();
    assert_eq!(props.len(), 9);
    assert_eq!(props[0].code, "BW");
}

#[test]
fn machine_names_and_property_codes_are_unique() {
    let c = ctx();
    let input = MachineInput {
        name: "pm1".into(),
        scanner_model: ScannerModel::Np1200,
        width_mm: 10_000.0,
        bin_count: Some(1000),
        speed_m_min: 1200.0,
        scan_time_s: 25.0,
    };
    assert_eq!(c.machines.create(input).unwrap_err().code, "conflict.machineNameExists");
    let property = PropertyInput {
        machine_id: pm1(&c),
        code: "bw".into(),
        name: "Basis Weight 2".into(),
        unit: "g/m²".into(),
        decimals: 2,
        sort_order: None,
        active: true,
        limits: Limits { target: 80.0, error_high: 84.0, warning_high: 82.0, warning_low: 78.0, error_low: 76.0 },
    };
    assert_eq!(c.properties.create(property).unwrap_err().code, "conflict.propertyCodeExists");
}

#[test]
fn generate_writes_one_file_per_property_outside_the_db() {
    let c = ctx();
    let id = pm1(&c);
    let reel = c.generator.generate(request(id, "PM1-001")).unwrap();
    assert_eq!(reel.files.len(), 9);
    assert_eq!(reel.scan_count, 60);
    let expected_dir = c.dir.path().join("PM1").join("PM1-001");
    assert_eq!(Path::new(&reel.directory), expected_dir);
    assert!(expected_dir.join("PM1-001_MOIST.csv").exists());

    let data = c.contour.load(reel.id.unwrap(), "BW").unwrap();
    assert_eq!(data.values.len(), 60);
    assert_eq!(data.values[0].len(), 800);
    assert_eq!(data.bin_centers.len(), 800);
    assert_eq!(data.md.len(), 60);
    assert!(data.summary.is_some());

    let again = c.contour.load_file(&reel.files[0].file_path).unwrap();
    assert_eq!(again.values, data.values);
}

#[test]
fn same_seed_reproduces_reel_and_overwrite_requires_confirmation() {
    let c = ctx();
    let id = pm1(&c);
    let first = c.generator.generate(request(id, "PM1-002")).unwrap();
    let a = c.contour.load(first.id.unwrap(), "CAL").unwrap();
    assert_eq!(c.generator.generate(request(id, "PM1-002")).unwrap_err().code, "conflict.reelExists");
    let second = c.generator.generate(GenerateRequest { overwrite: true, ..request(id, "PM1-002") }).unwrap();
    let b = c.contour.load(second.id.unwrap(), "CAL").unwrap();
    assert_eq!(a.values, b.values);
    assert_eq!(c.contour.list_reels(Some(id), 10).unwrap().len(), 1);
}

#[test]
fn duration_defines_scan_count_and_selected_properties() {
    let c = ctx();
    let id = pm1(&c);
    let props = c.properties.list(id).unwrap();
    let req = GenerateRequest {
        scan_count: None,
        duration_min: Some(10.0),
        property_ids: Some(vec![props[0].id.unwrap(), props[1].id.unwrap()]),
        ..request(id, "PM1-003")
    };
    let reel = c.generator.generate(req).unwrap();
    assert_eq!(reel.scan_count, 20);
    assert_eq!(reel.files.len(), 2);
}

#[test]
fn delete_reel_optionally_removes_files() {
    let c = ctx();
    let id = pm1(&c);
    let kept = c.generator.generate(request(id, "PM1-004")).unwrap();
    c.contour.delete_reel(kept.id.unwrap(), false).unwrap();
    assert!(Path::new(&kept.files[0].file_path).exists());

    let removed = c.generator.generate(GenerateRequest { overwrite: true, ..request(id, "PM1-005") }).unwrap();
    c.contour.delete_reel(removed.id.unwrap(), true).unwrap();
    assert!(!Path::new(&removed.directory).exists());
    assert!(c.contour.list_reels(None, 10).unwrap().is_empty());
}

#[test]
fn contour_colors_default_persist_and_validate() {
    let db = Database::open_in_memory().unwrap();
    let settings = SettingsService::new(Arc::new(SqliteSettingsRepository(db)), "exports".into());
    let current = settings.get().unwrap();
    assert_eq!(current.contour_colors, ContourColors::default());

    let colors = ContourColors { target: "#00FF00".into(), ..Default::default() };
    let saved = settings.update(Settings { contour_colors: colors, ..current.clone() }).unwrap();
    assert_eq!(saved.contour_colors.target, "#00ff00");
    assert_eq!(settings.get().unwrap().contour_colors.target, "#00ff00");

    let bad = ContourColors { error_low: "blue".into(), ..Default::default() };
    assert!(settings.update(Settings { contour_colors: bad, ..current }).is_err());
    assert_eq!(settings.get().unwrap().contour_colors.error_low, "#8381ff");
}
