use serde_path_to_error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExifToolError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("ExifTool error: {0}")]
    ExifTool(String),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Process terminated unexpectedly")]
    ProcessTerminated,

    #[error("stderr channel for exiftool disconnected.")]
    ChannelDisconnected,

    #[error("Operation timed out")]
    Timeout,

    #[error("Empty response, and no errors detected.")]
    EmptyResponse,

    #[error("Deserialization error at path '{path}': {source}")]
    Deserialization {
        path: String,
        source: serde_json::Error,
    },
}

impl From<serde_path_to_error::Error<serde_json::Error>> for ExifToolError {
    fn from(err: serde_path_to_error::Error<serde_json::Error>) -> Self {
        ExifToolError::Deserialization {
            path: err.path().to_string(),
            source: err.into_inner(),
        }
    }
}
