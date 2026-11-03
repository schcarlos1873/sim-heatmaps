use std::path::{Path, PathBuf};
use std::sync::Arc;

use domain::repositories::{ReelRecord, ReelRepository};
use domain::scan::Reel;
use domain::stats::{cd_profiles, md_series, reel_summary};
use domain::DomainError;

use crate::dto::{ContourData, ScanInfo};
use crate::error::logged;
use crate::ports::ReelStorage;
use crate::AppResult;

pub struct ContourService {
    reels: Arc<dyn ReelRepository>,
    storage: Arc<dyn ReelStorage>,
}

impl ContourService {
    pub fn new(reels: Arc<dyn ReelRepository>, storage: Arc<dyn ReelStorage>) -> Self {
        Self { reels, storage }
    }

    pub fn list_reels(&self, machine_id: Option<i64>, limit: u32) -> AppResult<Vec<ReelRecord>> {
        logged("contour.listReels", self.reels.list(machine_id, limit).map_err(Into::into))
    }

    /// Carrega o Contour Map de uma propriedade de uma bobina gerada.
    pub fn load(&self, reel_pk: i64, property_code: &str) -> AppResult<ContourData> {
        logged("contour.load", self.load_inner(reel_pk, property_code))
    }

    fn load_inner(&self, reel_pk: i64, property_code: &str) -> AppResult<ContourData> {
        let record = self.reels.get(reel_pk)?.ok_or(DomainError::NotFound { entity: "reel" })?;
        let file = record
            .files
            .iter()
            .find(|f| f.property_code.eq_ignore_ascii_case(property_code))
            .ok_or(DomainError::NotFound { entity: "property" })?;
        self.load_file_inner(Path::new(&file.file_path))
    }

    /// Abre um arquivo CSV escolhido pelo usuário (inclusive de outra instalação).
    pub fn load_file(&self, path: &str) -> AppResult<ContourData> {
        logged("contour.loadFile", self.load_file_inner(Path::new(path)))
    }

    fn load_file_inner(&self, path: &Path) -> AppResult<ContourData> {
        let reel = self.storage.read(path)?;
        Ok(to_contour(reel, path))
    }

    /// Remove a bobina do índice e, se pedido, os arquivos gerados (somente os listados no índice).
    pub fn delete_reel(&self, reel_pk: i64, delete_files: bool) -> AppResult<()> {
        logged("contour.deleteReel", self.delete_inner(reel_pk, delete_files))
    }

    fn delete_inner(&self, reel_pk: i64, delete_files: bool) -> AppResult<()> {
        let record = self.reels.get(reel_pk)?.ok_or(DomainError::NotFound { entity: "reel" })?;
        if delete_files {
            let files: Vec<PathBuf> = record.files.iter().map(|f| PathBuf::from(&f.file_path)).collect();
            self.storage.remove(&files, Path::new(&record.directory))?;
        }
        self.reels.delete(reel_pk)?;
        tracing::info!(reel = %record.reel_id, delete_files, "reel deleted");
        Ok(())
    }
}

fn to_contour(reel: Reel, path: &Path) -> ContourData {
    let bins = reel.machine.bin_count as usize;
    let cd = cd_profiles(&reel.scans, bins);
    let md = md_series(&reel.scans);
    let summary = reel_summary(&reel.scans);
    let bin_centers = reel.machine.bin_centers();
    let (scans, values) = reel
        .scans
        .into_iter()
        .map(|s| {
            let info = ScanInfo {
                number: s.number,
                direction: s.direction,
                started_at: s.started_at,
                ended_at: s.ended_at,
                speed: s.speed,
                md_position_m: s.md_position_m,
                edge_front_mm: s.edge_front_mm,
                edge_back_mm: s.edge_back_mm,
                sheet_break: s.sheet_break,
                sensor_status: s.sensor_status,
                sensor_temp_c: s.sensor_temp_c,
            };
            (info, s.values)
        })
        .unzip();
    ContourData {
        reel_id: reel.reel_id,
        file_path: path.to_string_lossy().into_owned(),
        machine: reel.machine,
        property: reel.property,
        bin_centers,
        scans,
        values,
        cd,
        md,
        summary,
    }
}
