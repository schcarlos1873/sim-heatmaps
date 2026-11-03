//! Camada de infraestrutura: SQLite (rusqlite), repositórios e arquivos CSV do scanner.

pub mod db;
pub mod repositories;
pub mod scan_csv;

pub use db::Database;
