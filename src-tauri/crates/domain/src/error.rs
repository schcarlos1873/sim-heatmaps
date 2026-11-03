use thiserror::Error;

/// Erros de regra de negócio. `code` é uma chave estável traduzida pela UI.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum DomainError {
    #[error("validation failed on '{field}': {code}")]
    Validation { field: &'static str, code: &'static str },
    #[error("{entity} not found")]
    NotFound { entity: &'static str },
    #[error("conflict: {code}")]
    Conflict { code: &'static str },
    #[error("storage error: {0}")]
    Storage(String),
}

impl DomainError {
    pub fn validation(field: &'static str, code: &'static str) -> Self {
        Self::Validation { field, code }
    }
}

pub type DomainResult<T> = Result<T, DomainError>;
