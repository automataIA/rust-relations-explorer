use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Regex match failed: {0}")]
    Regex(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid UTF-8 in file {file}")]
    InvalidUtf8 { file: PathBuf },
}

#[derive(Debug, Error)]
pub enum KnowledgeGraphError {
    #[error("Parse error in file {file}: {source}")]
    ParseError { file: PathBuf, source: ParseError },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid query: {0}")]
    Query(String),

    #[error("Visualization error: {0}")]
    Visualization(String),
}
