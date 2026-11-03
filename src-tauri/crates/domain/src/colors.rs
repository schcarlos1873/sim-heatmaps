use serde::{Deserialize, Serialize};

use crate::{DomainError, DomainResult};

/// Cores do Contour Map ancoradas nos limites (como o "Step color scaling" do OptiVision).
/// Entre dois limites vizinhos a cor é interpolada; além do Error a cor fica fixa.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContourColors {
    pub error_high: String,
    pub warning_high: String,
    pub target: String,
    pub warning_low: String,
    pub error_low: String,
}

/// Padrão extraído da escala de referência `assets/cores.png` (laranja → amarelo → verde-claro → azul-claro → lavanda).
pub const DEFAULT_CONTOUR_COLORS: [(&str, &str); 5] = [
    ("error_high", "#ff8e00"),
    ("warning_high", "#fae22e"),
    ("target", "#dbff8f"),
    ("warning_low", "#b3d7e5"),
    ("error_low", "#8381ff"),
];

impl Default for ContourColors {
    fn default() -> Self {
        let [eh, wh, t, wl, el] = DEFAULT_CONTOUR_COLORS.map(|(_, c)| c.to_string());
        Self { error_high: eh, warning_high: wh, target: t, warning_low: wl, error_low: el }
    }
}

fn is_hex_color(text: &str) -> bool {
    text.len() == 7 && text.starts_with('#') && text[1..].chars().all(|c| c.is_ascii_hexdigit())
}

impl ContourColors {
    /// Pares (chave de configuração, cor) na ordem Error High → Error Low.
    pub fn entries(&self) -> [(&'static str, &str); 5] {
        [
            ("error_high", &self.error_high),
            ("warning_high", &self.warning_high),
            ("target", &self.target),
            ("warning_low", &self.warning_low),
            ("error_low", &self.error_low),
        ]
    }

    /// Valida `#rrggbb` e normaliza para minúsculas.
    pub fn normalized(&self) -> DomainResult<Self> {
        let fix = |c: &str| {
            let c = c.trim().to_ascii_lowercase();
            if is_hex_color(&c) { Ok(c) } else { Err(DomainError::validation("contourColors", "invalidColor")) }
        };
        Ok(Self {
            error_high: fix(&self.error_high)?,
            warning_high: fix(&self.warning_high)?,
            target: fix(&self.target)?,
            warning_low: fix(&self.warning_low)?,
            error_low: fix(&self.error_low)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        let c = ContourColors::default();
        assert_eq!(c.normalized(), Ok(c.clone()));
        assert_eq!(c.error_high, "#ff8e00");
    }

    #[test]
    fn validates_and_normalizes_hex() {
        let c = ContourColors { target: " #DBFF8F ".into(), ..Default::default() };
        assert_eq!(c.normalized().unwrap().target, "#dbff8f");
        for bad in ["dbff8f", "#dbff8", "#gggggg", "red"] {
            let c = ContourColors { target: bad.into(), ..Default::default() };
            assert!(c.normalized().is_err(), "{bad}");
        }
    }
}
