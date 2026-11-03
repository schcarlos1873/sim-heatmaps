use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::NaiveDateTime;
use domain::generator::{generate_frames, generate_reel, UniformSource};
use domain::repositories::{MachineRepository, PropertyRepository, ReelFile, ReelRecord, ReelRepository};
use domain::scan::validate_reel_id;
use domain::DomainError;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::dto::GenerateRequest;
use crate::error::logged;
use crate::ports::ReelStorage;
use crate::services::SettingsService;
use crate::AppResult;

struct Rand(StdRng);

impl UniformSource for Rand {
    fn next_f64(&mut self) -> f64 {
        self.0.gen::<f64>()
    }
}

const DATE_FORMATS: [&str; 2] = ["%Y-%m-%dT%H:%M:%S", "%Y-%m-%dT%H:%M"];

pub struct GeneratorService {
    machines: Arc<dyn MachineRepository>,
    properties: Arc<dyn PropertyRepository>,
    reels: Arc<dyn ReelRepository>,
    storage: Arc<dyn ReelStorage>,
    settings: Arc<SettingsService>,
}

impl GeneratorService {
    pub fn new(
        machines: Arc<dyn MachineRepository>,
        properties: Arc<dyn PropertyRepository>,
        reels: Arc<dyn ReelRepository>,
        storage: Arc<dyn ReelStorage>,
        settings: Arc<SettingsService>,
    ) -> Self {
        Self { machines, properties, reels, storage, settings }
    }

    /// Gera uma bobina: um CSV por propriedade em `<export_dir>/<máquina>/<reel_id>/`.
    pub fn generate(&self, request: GenerateRequest) -> AppResult<ReelRecord> {
        logged("generator.generate", self.generate_inner(request))
    }

    fn generate_inner(&self, request: GenerateRequest) -> AppResult<ReelRecord> {
        let reel_id = request.reel_id.trim().to_string();
        validate_reel_id(&reel_id)?;
        let started_at = parse_datetime(&request.started_at)?;
        let machine = self.machines.get(request.machine_id)?.ok_or(DomainError::NotFound { entity: "machine" })?;
        let scan_count = match (request.scan_count, request.duration_min) {
            (Some(n), _) => n,
            (None, Some(d)) if d.is_finite() && d > 0.0 => (d * 60.0 / machine.scan_time_s).ceil() as u32,
            (None, _) => return Err(DomainError::validation("durationMin", "positive").into()),
        };

        let properties: Vec<_> = self
            .properties
            .list_by_machine(machine.id.unwrap_or_default())?
            .into_iter()
            .filter(|p| match &request.property_ids {
                Some(ids) => p.id.is_some_and(|id| ids.contains(&id)),
                None => p.active,
            })
            .collect();
        if properties.is_empty() {
            return Err(DomainError::validation("properties", "required").into());
        }

        let previous = self.reels.find(&machine.name, &reel_id)?;
        let dir = Path::new(&self.settings.get()?.export_dir).join(&machine.name).join(&reel_id);
        let dir_has_files = std::fs::read_dir(&dir).map(|mut d| d.next().is_some()).unwrap_or(false);
        if !request.overwrite && (previous.is_some() || dir_has_files) {
            return Err(DomainError::Conflict { code: "reelExists" }.into());
        }
        if let Some(prev) = &previous {
            let files: Vec<PathBuf> = prev.files.iter().map(|f| PathBuf::from(&f.file_path)).collect();
            self.storage.remove(&files, Path::new(&prev.directory))?;
        }

        let mut rng = Rand(match request.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        });
        let frames = generate_frames(&machine, started_at, scan_count, &request.params, &mut rng)?;
        let mut files = Vec::with_capacity(properties.len());
        for p in &properties {
            let reel = generate_reel(&reel_id, &machine, p, &frames, &request.params, &mut rng)?;
            let path = self.storage.write(&reel, &dir)?;
            files.push(ReelFile {
                property_code: p.code.clone(),
                property_name: p.name.clone(),
                file_path: path.to_string_lossy().into_owned(),
            });
        }

        let mut record = ReelRecord {
            id: None,
            reel_id,
            machine_id: machine.id,
            machine_name: machine.name.clone(),
            started_at: frames.first().map(|f| f.started_at).unwrap_or(started_at),
            ended_at: frames.last().map(|f| f.ended_at).unwrap_or(started_at),
            scan_count,
            bin_count: machine.bin_count,
            directory: dir.to_string_lossy().into_owned(),
            files,
            created_at: None,
        };
        record.id = Some(self.reels.upsert(&record)?);
        tracing::info!(reel = %record.reel_id, machine = %machine.name, scans = scan_count, files = record.files.len(), "reel generated");
        Ok(record)
    }
}

fn parse_datetime(text: &str) -> AppResult<NaiveDateTime> {
    DATE_FORMATS
        .iter()
        .find_map(|f| NaiveDateTime::parse_from_str(text.trim(), f).ok())
        .ok_or_else(|| DomainError::validation("startedAt", "invalidDate").into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_datetime_local() {
        assert!(parse_datetime("2026-11-02T08:00").is_ok());
        assert!(parse_datetime("2026-11-02T08:00:30").is_ok());
        assert!(parse_datetime("02/11/2026").is_err());
    }
}
