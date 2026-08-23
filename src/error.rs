use thiserror::Error;

#[derive(Error, Debug)]
pub enum SiphonixError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Engine error: {0}")]
    Engine(String),

    #[error("Process execution error: {0}")]
    Process(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Metadata inspection error: {0}")]
    Inspection(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, SiphonixError>;
