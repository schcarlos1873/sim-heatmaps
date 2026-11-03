use std::sync::Arc;

use domain::colors::ContourColors;
use domain::repositories::SettingsRepository;
use domain::DomainError;

use crate::dto::Settings;
use crate::error::logged;
use crate::AppResult;

const EXPORT_DIR: &str = "export_dir";
/// Prefixo das chaves das cores do Contour Map (ex.: `contour_color.error_high`).
const COLOR_PREFIX: &str = "contour_color.";

pub struct SettingsService {
    repo: Arc<dyn SettingsRepository>,
    default_export_dir: String,
}

impl SettingsService {
    pub fn new(repo: Arc<dyn SettingsRepository>, default_export_dir: String) -> Self {
        Self { repo, default_export_dir }
    }

    pub fn get(&self) -> AppResult<Settings> {
        logged("settings.get", self.get_inner())
    }

    fn get_inner(&self) -> AppResult<Settings> {
        let defaults = ContourColors::default();
        let color = |key: &str, default: &str| -> AppResult<String> {
            Ok(self.repo.get(&format!("{COLOR_PREFIX}{key}"))?.unwrap_or_else(|| default.to_string()))
        };
        let contour_colors = ContourColors {
            error_high: color("error_high", &defaults.error_high)?,
            warning_high: color("warning_high", &defaults.warning_high)?,
            target: color("target", &defaults.target)?,
            warning_low: color("warning_low", &defaults.warning_low)?,
            error_low: color("error_low", &defaults.error_low)?,
        };
        Ok(Settings {
            export_dir: self.repo.get(EXPORT_DIR)?.unwrap_or_else(|| self.default_export_dir.clone()),
            contour_colors,
        })
    }

    pub fn update(&self, settings: Settings) -> AppResult<Settings> {
        logged("settings.update", self.update_inner(settings))
    }

    fn update_inner(&self, settings: Settings) -> AppResult<Settings> {
        let export_dir = settings.export_dir.trim();
        if export_dir.is_empty() {
            return Err(DomainError::validation("exportDir", "required").into());
        }
        let colors = settings.contour_colors.normalized()?;
        self.repo.set(EXPORT_DIR, export_dir)?;
        for (key, value) in colors.entries() {
            self.repo.set(&format!("{COLOR_PREFIX}{key}"), value)?;
        }
        self.get_inner()
    }
}
