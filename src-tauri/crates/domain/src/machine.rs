use serde::{Deserialize, Serialize};

use crate::{DomainError, DomainResult};

/// Modelo do scanner instalado na máquina. Apenas cadastro: não altera a geração dos dados.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScannerModel {
    #[serde(rename = "NP800")]
    Np800,
    #[serde(rename = "NP1200")]
    Np1200,
    #[serde(rename = "RNP1200")]
    Rnp1200,
}

impl ScannerModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Np800 => "NP800",
            Self::Np1200 => "NP1200",
            Self::Rnp1200 => "RNP1200",
        }
    }

    pub fn parse(text: &str) -> DomainResult<Self> {
        match text {
            "NP800" => Ok(Self::Np800),
            "NP1200" => Ok(Self::Np1200),
            "RNP1200" => Ok(Self::Rnp1200),
            _ => Err(DomainError::validation("scannerModel", "invalid")),
        }
    }
}

/// Máquina de papel com scanner de perfil. A largura da folha é dividida em `bin_count` data bins.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Machine {
    pub id: Option<i64>,
    pub name: String,
    pub scanner_model: ScannerModel,
    /// Largura da folha medida pelo scanner (mm).
    pub width_mm: f64,
    pub bin_count: u32,
    /// Velocidade nominal da máquina (m/min).
    pub speed_m_min: f64,
    /// Duração de uma travessia do scanner (s).
    pub scan_time_s: f64,
}

pub const DEFAULT_BIN_COUNT: u32 = 800;
pub const MIN_BINS: u32 = 10;
pub const MAX_BINS: u32 = 2000;
pub const MAX_WIDTH_MM: f64 = 15_000.0;

impl Machine {
    pub fn validate(&self) -> DomainResult<()> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(DomainError::validation("name", "required"));
        }
        if name.len() > 32 || !name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_')) {
            return Err(DomainError::validation("name", "invalid"));
        }
        if !self.width_mm.is_finite() || self.width_mm <= 0.0 || self.width_mm > MAX_WIDTH_MM {
            return Err(DomainError::validation("widthMm", "range"));
        }
        if !(MIN_BINS..=MAX_BINS).contains(&self.bin_count) {
            return Err(DomainError::validation("binCount", "range"));
        }
        if !self.speed_m_min.is_finite() || self.speed_m_min <= 0.0 {
            return Err(DomainError::validation("speedMMin", "positive"));
        }
        if !self.scan_time_s.is_finite() || self.scan_time_s <= 0.0 {
            return Err(DomainError::validation("scanTimeS", "positive"));
        }
        Ok(())
    }

    /// Largura de cada data bin (mm): os pontos acompanham a largura da máquina.
    pub fn bin_width(&self) -> f64 {
        self.width_mm / self.bin_count as f64
    }

    /// Posição CD do centro de cada bin, a partir da borda frontal (mm).
    pub fn bin_centers(&self) -> Vec<f64> {
        let w = self.bin_width();
        (0..self.bin_count).map(|i| w * (i as f64 + 0.5)).collect()
    }

    /// Avanço da folha durante um scan (m).
    pub fn md_per_scan_m(&self) -> f64 {
        self.speed_m_min * self.scan_time_s / 60.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pm1() -> Machine {
        Machine {
            id: None,
            name: "PM1".into(),
            scanner_model: ScannerModel::Np800,
            width_mm: 8000.0,
            bin_count: DEFAULT_BIN_COUNT,
            speed_m_min: 900.0,
            scan_time_s: 30.0,
        }
    }

    #[test]
    fn bins_follow_machine_width() {
        let m = pm1();
        assert_eq!(m.bin_width(), 10.0);
        let c = m.bin_centers();
        assert_eq!(c.len(), 800);
        assert_eq!(c[0], 5.0);
        assert_eq!(c[799], 7995.0);
        assert_eq!(m.md_per_scan_m(), 450.0);
    }

    #[test]
    fn rejects_invalid() {
        assert!(pm1().validate().is_ok());
        assert!(Machine { name: " ".into(), ..pm1() }.validate().is_err());
        assert!(Machine { name: "PM 1".into(), ..pm1() }.validate().is_err());
        assert!(Machine { bin_count: 5, ..pm1() }.validate().is_err());
        assert!(Machine { width_mm: 0.0, ..pm1() }.validate().is_err());
        assert!(Machine { scan_time_s: 0.0, ..pm1() }.validate().is_err());
    }

    #[test]
    fn parses_scanner_model() {
        assert_eq!(ScannerModel::parse("RNP1200"), Ok(ScannerModel::Rnp1200));
        assert!(ScannerModel::parse("X").is_err());
    }
}
