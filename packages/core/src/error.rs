use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Serial port error: {0}")]
    SerialError(String),

    #[error("Failed to open port: {0}")]
    OpenError(String),

    #[error("Failed to close port: {0}")]
    CloseError(String),

    #[error("Read error: {0}")]
    ReadError(String),

    #[error("Write error: {0}")]
    WriteError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
