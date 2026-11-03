use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::machine::Machine;
use crate::property::Property;
use crate::{DomainError, DomainResult};

/// Sentido da travessia do scanner: da borda frontal (FS) para a traseira (BS) ou o inverso.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScanDirection {
    Forward,
    Reverse,
}

impl ScanDirection {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Forward => "F",
            Self::Reverse => "R",
        }
    }

    pub fn parse(text: &str) -> DomainResult<Self> {
        match text {
            "F" => Ok(Self::Forward),
            "R" => Ok(Self::Reverse),
            _ => Err(DomainError::validation("direction", "invalid")),
        }
    }
}

/// Status de validação do sensor durante a passagem. Diferente de `Ok`, o scan não tem valores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SensorStatus {
    Ok,
    NotCalibrated,
    Dirty,
    Fault,
}

impl SensorStatus {
    pub fn code(&self) -> u8 {
        match self {
            Self::Ok => 0,
            Self::NotCalibrated => 1,
            Self::Dirty => 2,
            Self::Fault => 3,
        }
    }

    pub fn from_code(code: u8) -> DomainResult<Self> {
        match code {
            0 => Ok(Self::Ok),
            1 => Ok(Self::NotCalibrated),
            2 => Ok(Self::Dirty),
            3 => Ok(Self::Fault),
            _ => Err(DomainError::validation("sensorStatus", "invalid")),
        }
    }
}

/// Uma travessia do scanner: metadados de posição/sincronismo, diagnóstico e o perfil CD.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scan {
    pub number: u32,
    pub direction: ScanDirection,
    pub started_at: NaiveDateTime,
    pub ended_at: NaiveDateTime,
    /// Velocidade da máquina durante o scan (m/min).
    pub speed: f64,
    /// Comprimento produzido no fim do scan (m).
    pub md_position_m: f64,
    /// Bordas da folha detectadas (mm a partir da borda frontal).
    pub edge_front_mm: f64,
    pub edge_back_mm: f64,
    pub sheet_break: bool,
    pub sensor_status: SensorStatus,
    pub sensor_temp_c: f64,
    /// Um valor por bin. `None` fora das bordas, em quebra de folha ou com sensor inválido.
    pub values: Vec<Option<f64>>,
}

impl Scan {
    pub fn is_valid(&self) -> bool {
        !self.sheet_break && self.sensor_status == SensorStatus::Ok
    }
}

/// Bobina (reel) de uma propriedade: matriz CD × MD com snapshot da máquina e dos limites.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reel {
    pub reel_id: String,
    pub machine: Machine,
    pub property: Property,
    pub scans: Vec<Scan>,
}

pub const MAX_SCANS: u32 = 2880;

/// O Reel ID compõe o nome dos arquivos: apenas letras, dígitos, '-', '_' e '.'.
pub fn validate_reel_id(reel_id: &str) -> DomainResult<()> {
    if reel_id.is_empty() {
        return Err(DomainError::validation("reelId", "required"));
    }
    if reel_id.len() > 64
        || reel_id.starts_with('.')
        || !reel_id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Err(DomainError::validation("reelId", "invalidChars"));
    }
    Ok(())
}

impl Reel {
    pub fn validate(&self) -> DomainResult<()> {
        validate_reel_id(&self.reel_id)?;
        if self.scans.is_empty() || self.scans.len() > MAX_SCANS as usize {
            return Err(DomainError::validation("scans", "range"));
        }
        let bins = self.machine.bin_count as usize;
        for s in &self.scans {
            if s.values.len() != bins {
                return Err(DomainError::validation("scans", "binCount"));
            }
            if s.values.iter().flatten().any(|v| !v.is_finite()) {
                return Err(DomainError::validation("scans", "number"));
            }
        }
        Ok(())
    }

    pub fn started_at(&self) -> Option<NaiveDateTime> {
        self.scans.first().map(|s| s.started_at)
    }

    pub fn ended_at(&self) -> Option<NaiveDateTime> {
        self.scans.last().map(|s| s.ended_at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_reel_id() {
        assert!(validate_reel_id("PM1-20261102-01").is_ok());
        assert!(validate_reel_id("").is_err());
        assert!(validate_reel_id("../x").is_err());
        assert!(validate_reel_id("a b").is_err());
    }

    #[test]
    fn codes_round_trip() {
        for s in [SensorStatus::Ok, SensorStatus::NotCalibrated, SensorStatus::Dirty, SensorStatus::Fault] {
            assert_eq!(SensorStatus::from_code(s.code()), Ok(s));
        }
        assert_eq!(ScanDirection::parse(ScanDirection::Reverse.code()), Ok(ScanDirection::Reverse));
    }
}
