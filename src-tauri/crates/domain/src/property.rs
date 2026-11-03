use serde::{Deserialize, Serialize};

use crate::{DomainError, DomainResult};

/// Limites de uma propriedade. Ordem obrigatória:
/// `ErrorLow < WarningLow < Target < WarningHigh < ErrorHigh`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Limits {
    pub target: f64,
    pub error_high: f64,
    pub warning_high: f64,
    pub warning_low: f64,
    pub error_low: f64,
}

/// Classificação de um valor em relação aos limites (verde / amarelo / vermelho).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LimitStatus {
    Ok,
    Warning,
    Error,
}

impl Limits {
    pub fn validate(&self) -> DomainResult<()> {
        let all = [self.error_low, self.warning_low, self.target, self.warning_high, self.error_high];
        if all.iter().any(|v| !v.is_finite()) {
            return Err(DomainError::validation("limits", "number"));
        }
        if self.error_low >= self.warning_low {
            return Err(DomainError::validation("errorLow", "lessThanWarningLow"));
        }
        if self.warning_low >= self.target {
            return Err(DomainError::validation("warningLow", "lessThanTarget"));
        }
        if self.target >= self.warning_high {
            return Err(DomainError::validation("warningHigh", "greaterThanTarget"));
        }
        if self.warning_high >= self.error_high {
            return Err(DomainError::validation("errorHigh", "greaterThanWarningHigh"));
        }
        Ok(())
    }

    pub fn classify(&self, value: f64) -> LimitStatus {
        if value < self.error_low || value > self.error_high {
            LimitStatus::Error
        } else if value < self.warning_low || value > self.warning_high {
            LimitStatus::Warning
        } else {
            LimitStatus::Ok
        }
    }

    /// Menor distância entre o Target e um limite de Warning.
    pub fn warning_half_band(&self) -> f64 {
        (self.warning_high - self.target).min(self.target - self.warning_low)
    }
}

/// Propriedade medida pelo scanner (Basis Weight, Moisture, Caliper, L*, ...) com limites por máquina.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Property {
    pub id: Option<i64>,
    pub machine_id: i64,
    /// Código curto usado no nome do arquivo (ex.: `BW`, `MOIST`).
    pub code: String,
    pub name: String,
    pub unit: String,
    pub decimals: u8,
    pub sort_order: i32,
    pub active: bool,
    pub limits: Limits,
}

pub const MAX_DECIMALS: u8 = 6;

impl Property {
    pub fn validate(&self) -> DomainResult<()> {
        let code = self.code.trim();
        if code.is_empty() {
            return Err(DomainError::validation("code", "required"));
        }
        if code.len() > 16 || !code.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_')) {
            return Err(DomainError::validation("code", "invalid"));
        }
        if self.name.trim().is_empty() {
            return Err(DomainError::validation("name", "required"));
        }
        if self.name.len() > 64 || self.name.contains(['\r', '\n', ';']) {
            return Err(DomainError::validation("name", "invalid"));
        }
        if self.unit.len() > 16 || self.unit.contains(['\r', '\n', ';']) {
            return Err(DomainError::validation("unit", "invalid"));
        }
        if self.decimals > MAX_DECIMALS {
            return Err(DomainError::validation("decimals", "range"));
        }
        self.limits.validate()
    }
}

/// Modelo de propriedade padrão criado junto com uma nova máquina.
/// Os limites são ilustrativos e devem ser ajustados aos valores reais de cada máquina.
pub struct PropertyTemplate {
    pub code: &'static str,
    pub name: &'static str,
    pub unit: &'static str,
    pub decimals: u8,
    /// target, warning (±), error (±)
    pub target: f64,
    pub warning: f64,
    pub error: f64,
}

pub const DEFAULT_PROPERTIES: [PropertyTemplate; 9] = [
    PropertyTemplate { code: "BW", name: "Basis Weight", unit: "g/m²", decimals: 2, target: 80.0, warning: 2.0, error: 4.0 },
    PropertyTemplate { code: "MOIST", name: "Moisture", unit: "%", decimals: 2, target: 7.0, warning: 0.5, error: 1.0 },
    PropertyTemplate { code: "CAL", name: "Caliper", unit: "µm", decimals: 1, target: 105.0, warning: 3.0, error: 6.0 },
    PropertyTemplate { code: "ASH", name: "Ash", unit: "%", decimals: 2, target: 18.0, warning: 1.0, error: 2.0 },
    PropertyTemplate { code: "L", name: "L*", unit: "", decimals: 2, target: 95.0, warning: 0.5, error: 1.0 },
    PropertyTemplate { code: "A", name: "a*", unit: "", decimals: 2, target: 1.0, warning: 0.3, error: 0.6 },
    PropertyTemplate { code: "B", name: "b*", unit: "", decimals: 2, target: -4.0, warning: 0.4, error: 0.8 },
    PropertyTemplate { code: "BRIGHT", name: "Brightness", unit: "%ISO", decimals: 2, target: 92.0, warning: 0.8, error: 1.6 },
    PropertyTemplate { code: "OPAC", name: "Opacity", unit: "%", decimals: 2, target: 90.0, warning: 0.8, error: 1.6 },
];

impl PropertyTemplate {
    pub fn to_property(&self, machine_id: i64, sort_order: i32) -> Property {
        Property {
            id: None,
            machine_id,
            code: self.code.into(),
            name: self.name.into(),
            unit: self.unit.into(),
            decimals: self.decimals,
            sort_order,
            active: true,
            limits: Limits {
                target: self.target,
                error_high: self.target + self.error,
                warning_high: self.target + self.warning,
                warning_low: self.target - self.warning,
                error_low: self.target - self.error,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> Limits {
        Limits { target: 10.0, error_high: 13.0, warning_high: 12.0, warning_low: 8.0, error_low: 7.0 }
    }

    #[test]
    fn classifies_values() {
        let l = limits();
        assert_eq!(l.classify(10.0), LimitStatus::Ok);
        assert_eq!(l.classify(12.0), LimitStatus::Ok);
        assert_eq!(l.classify(12.5), LimitStatus::Warning);
        assert_eq!(l.classify(7.5), LimitStatus::Warning);
        assert_eq!(l.classify(13.1), LimitStatus::Error);
        assert_eq!(l.classify(6.0), LimitStatus::Error);
    }

    #[test]
    fn enforces_limit_order() {
        assert!(limits().validate().is_ok());
        let mut l = limits();
        l.warning_low = 11.0;
        assert_eq!(l.validate(), Err(DomainError::validation("warningLow", "lessThanTarget")));
    }

    #[test]
    fn default_properties_are_valid() {
        for (i, t) in DEFAULT_PROPERTIES.iter().enumerate() {
            assert!(t.to_property(1, i as i32).validate().is_ok(), "{}", t.code);
        }
    }

    #[test]
    fn validates_code() {
        let mut p = DEFAULT_PROPERTIES[0].to_property(1, 0);
        p.code = "B W".into();
        assert_eq!(p.validate(), Err(DomainError::validation("code", "invalid")));
    }
}
