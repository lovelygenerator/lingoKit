//! The library (sample: counting the words of a text file).

use std::path::{Path, PathBuf};

// Defines LOCALES (translations) and msg (typed accessors), also used by the CLI and the MCP server.
lingokit::include_locales!();

#[derive(Debug, thiserror::Error, lingokit::LocalizedError)]
pub enum Error {
    #[error("file not found: {}", path.display())]
    #[ftl("err-not-found")]
    NotFound { path: PathBuf },

    #[error("file is empty: {}", path.display())]
    #[ftl("err-empty")]
    Empty { path: PathBuf },

    #[error("failed to read {}", path.display())]
    #[ftl("err-io")]
    Io { path: PathBuf, source: std::io::Error },
}

pub type Result<T> = std::result::Result<T, Error>;

/// Counts the words of a text file.
pub fn count_words(path: &Path) -> Result<usize> {
    let text = std::fs::read_to_string(path).map_err(|source| match source.kind() {
        std::io::ErrorKind::NotFound => Error::NotFound { path: path.into() },
        _ => Error::Io { path: path.into(), source },
    })?;
    if text.trim().is_empty() {
        return Err(Error::Empty { path: path.into() });
    }
    Ok(text.split_whitespace().count())
}
