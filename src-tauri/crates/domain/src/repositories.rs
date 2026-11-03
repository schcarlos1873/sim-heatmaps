//! Contratos de persistência. Implementados na camada de infraestrutura.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::machine::Machine;
use crate::property::Property;
use crate::DomainResult;

pub trait MachineRepository: Send + Sync {
    fn list(&self) -> DomainResult<Vec<Machine>>;
    fn get(&self, id: i64) -> DomainResult<Option<Machine>>;
    fn find_by_name(&self, name: &str) -> DomainResult<Option<Machine>>;
    fn insert(&self, machine: &Machine) -> DomainResult<i64>;
    fn update(&self, machine: &Machine) -> DomainResult<()>;
    fn delete(&self, id: i64) -> DomainResult<()>;
    fn count(&self) -> DomainResult<i64>;
}

pub trait PropertyRepository: Send + Sync {
    fn list_by_machine(&self, machine_id: i64) -> DomainResult<Vec<Property>>;
    fn get(&self, id: i64) -> DomainResult<Option<Property>>;
    fn find_by_code(&self, machine_id: i64, code: &str) -> DomainResult<Option<Property>>;
    fn insert(&self, property: &Property) -> DomainResult<i64>;
    fn update(&self, property: &Property) -> DomainResult<()>;
    fn delete(&self, id: i64) -> DomainResult<()>;
}

/// Arquivo gerado para uma propriedade da bobina.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReelFile {
    pub property_code: String,
    pub property_name: String,
    pub file_path: String,
}

/// Índice de uma bobina gerada. Os valores ficam somente nos arquivos.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReelRecord {
    pub id: Option<i64>,
    pub reel_id: String,
    pub machine_id: Option<i64>,
    pub machine_name: String,
    pub started_at: NaiveDateTime,
    pub ended_at: NaiveDateTime,
    pub scan_count: u32,
    pub bin_count: u32,
    pub directory: String,
    pub files: Vec<ReelFile>,
    pub created_at: Option<NaiveDateTime>,
}

pub trait ReelRepository: Send + Sync {
    fn list(&self, machine_id: Option<i64>, limit: u32) -> DomainResult<Vec<ReelRecord>>;
    fn get(&self, id: i64) -> DomainResult<Option<ReelRecord>>;
    fn find(&self, machine_name: &str, reel_id: &str) -> DomainResult<Option<ReelRecord>>;
    /// Insere a bobina; se já existir (mesma máquina + Reel ID) substitui.
    fn upsert(&self, reel: &ReelRecord) -> DomainResult<i64>;
    fn delete(&self, id: i64) -> DomainResult<()>;
}

pub trait SettingsRepository: Send + Sync {
    fn get(&self, key: &str) -> DomainResult<Option<String>>;
    fn set(&self, key: &str, value: &str) -> DomainResult<()>;
}
