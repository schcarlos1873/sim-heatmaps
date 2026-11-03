//! Camada de aplicação: casos de uso que orquestram domínio e portas de infraestrutura.

pub mod dto;
pub mod error;
pub mod ports;
pub mod services;

pub use error::{AppError, AppResult};
