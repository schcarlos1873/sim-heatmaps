//! Portas implementadas pela infraestrutura (arquivos CSV do scanner).

use std::path::{Path, PathBuf};

use domain::scan::Reel;

use crate::AppResult;

pub trait ReelStorage: Send + Sync {
    /// Grava `<dir>/<reel_id>_<code>.csv`, substituindo o arquivo se existir.
    fn write(&self, reel: &Reel, dir: &Path) -> AppResult<PathBuf>;
    /// Lê um arquivo gravado por `write`. Máquina e propriedade vêm do preâmbulo (sem `id`).
    fn read(&self, path: &Path) -> AppResult<Reel>;
    /// Remove os arquivos indicados e a pasta, se ficar vazia.
    fn remove(&self, files: &[PathBuf], dir: &Path) -> AppResult<()>;
}
