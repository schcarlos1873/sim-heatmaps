use domain::DomainError;
use serde::Serialize;

/// Erro devolvido à UI. `code` é uma chave i18n (ex.: `validation.required`).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub field: Option<String>,
    pub message: String,
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self { code: code.into(), field: None, message: message.into() }
    }

    pub fn io(err: impl std::fmt::Display) -> Self {
        Self::new("io", err.to_string())
    }

    /// Falhas inesperadas (I/O, banco) — registradas como `error` no log.
    pub fn is_internal(&self) -> bool {
        matches!(self.code.as_str(), "io" | "storage" | "parse")
    }
}

impl From<DomainError> for AppError {
    fn from(err: DomainError) -> Self {
        let message = err.to_string();
        match err {
            DomainError::Validation { field, code } => Self {
                code: format!("validation.{code}"),
                field: Some(field.to_string()),
                message,
            },
            DomainError::NotFound { entity } => Self::new(format!("notFound.{entity}"), message),
            DomainError::Conflict { code } => Self::new(format!("conflict.{code}"), message),
            DomainError::Storage(_) => Self::new("storage", message),
        }
    }
}

/// Registra o erro do caso de uso no log (pasta /log) e o devolve inalterado.
pub fn logged<T>(operation: &str, result: AppResult<T>) -> AppResult<T> {
    if let Err(e) = &result {
        if e.is_internal() {
            tracing::error!(operation, code = %e.code, "{}", e.message);
        } else {
            tracing::warn!(operation, code = %e.code, field = ?e.field, "{}", e.message);
        }
    }
    result
}
