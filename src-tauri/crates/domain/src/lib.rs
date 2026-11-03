//! Camada de domínio: entidades, regras de negócio e contratos de repositório.
//! Não depende de nenhuma outra camada nem de I/O.

pub mod colors;
pub mod error;
pub mod format;
pub mod generator;
pub mod machine;
pub mod property;
pub mod repositories;
pub mod scan;
pub mod stats;

pub use error::{DomainError, DomainResult};
