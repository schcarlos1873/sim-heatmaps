use std::sync::Arc;

use domain::machine::{Machine, ScannerModel, DEFAULT_BIN_COUNT};
use domain::property::DEFAULT_PROPERTIES;
use domain::repositories::{MachineRepository, PropertyRepository};
use domain::DomainError;

use crate::dto::MachineInput;
use crate::error::logged;
use crate::AppResult;

pub struct MachineService {
    machines: Arc<dyn MachineRepository>,
    properties: Arc<dyn PropertyRepository>,
}

impl MachineService {
    pub fn new(machines: Arc<dyn MachineRepository>, properties: Arc<dyn PropertyRepository>) -> Self {
        Self { machines, properties }
    }

    pub fn list(&self) -> AppResult<Vec<Machine>> {
        logged("machines.list", self.machines.list().map_err(Into::into))
    }

    /// Cria a máquina com o conjunto padrão de propriedades (limites ilustrativos, ajustáveis).
    pub fn create(&self, input: MachineInput) -> AppResult<Machine> {
        logged("machines.create", self.create_inner(input))
    }

    fn create_inner(&self, input: MachineInput) -> AppResult<Machine> {
        let mut machine = build(None, input);
        machine.validate()?;
        if self.machines.find_by_name(&machine.name)?.is_some() {
            return Err(DomainError::Conflict { code: "machineNameExists" }.into());
        }
        let id = self.machines.insert(&machine)?;
        machine.id = Some(id);
        for (i, t) in DEFAULT_PROPERTIES.iter().enumerate() {
            self.properties.insert(&t.to_property(id, i as i32 + 1))?;
        }
        tracing::info!(machine = %machine.name, bins = machine.bin_count, "machine created");
        Ok(machine)
    }

    /// Altera largura/bins sem afetar bobinas já geradas (cada arquivo guarda o snapshot da máquina).
    pub fn update(&self, id: i64, input: MachineInput) -> AppResult<Machine> {
        logged("machines.update", self.update_inner(id, input))
    }

    fn update_inner(&self, id: i64, input: MachineInput) -> AppResult<Machine> {
        self.machines.get(id)?.ok_or(DomainError::NotFound { entity: "machine" })?;
        let machine = build(Some(id), input);
        machine.validate()?;
        if let Some(other) = self.machines.find_by_name(&machine.name)? {
            if other.id != Some(id) {
                return Err(DomainError::Conflict { code: "machineNameExists" }.into());
            }
        }
        self.machines.update(&machine)?;
        Ok(machine)
    }

    pub fn delete(&self, id: i64) -> AppResult<()> {
        logged("machines.delete", self.machines.delete(id).map_err(Into::into))
    }

    /// Primeira execução: cria a máquina PM1 (NP800, 8 m, 800 bins). Retorna `true` se criou.
    pub fn seed_if_empty(&self) -> AppResult<bool> {
        if self.machines.count()? > 0 {
            return Ok(false);
        }
        self.create(MachineInput {
            name: "PM1".into(),
            scanner_model: ScannerModel::Np800,
            width_mm: 8000.0,
            bin_count: None,
            speed_m_min: 900.0,
            scan_time_s: 30.0,
        })?;
        Ok(true)
    }
}

fn build(id: Option<i64>, input: MachineInput) -> Machine {
    Machine {
        id,
        name: input.name.trim().to_string(),
        scanner_model: input.scanner_model,
        width_mm: input.width_mm,
        bin_count: input.bin_count.unwrap_or(DEFAULT_BIN_COUNT),
        speed_m_min: input.speed_m_min,
        scan_time_s: input.scan_time_s,
    }
}
