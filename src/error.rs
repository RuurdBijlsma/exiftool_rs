use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur when interacting with ExifTool.
#[derive(Debug, Error)]
pub enum ExifToolError {
    #[error("ExifTool executable not found or failed to start: {0}")]
    ExifToolNotFound(#[source] std::io::Error),

    #[error("IO error communicating with ExifTool: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("File not found: '{path}'. Command: exiftool {command_args}")]
    FileNotFound { path: PathBuf, command_args: String },

    #[error(
        "ExifTool process error: {message}. Command: exiftool {command_args}, std_err: {std_err}"
    )]
    ExifToolProcess {
        message: String,
        std_err: String,
        command_args: String,
    },

    #[error("ExifTool process terminated unexpectedly.")]
    ProcessTerminated,

    #[error("ExifTool stderr stream disconnected.")]
    StderrDisconnected,

    #[error("Received unexpected output format from ExifTool for file '{path}'. Command: exiftool {command_args}")]
    UnexpectedFormat { path: String, command_args: String },

    #[error("Tag '{tag}' not found in metadata for file '{path}'.")]
    TagNotFound { path: PathBuf, tag: String },

    #[error("Deserialization error at path '{path}': {source}")]
    Deserialization {
        path: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("Failed to deserialize tag '{tag}' for file '{path}': {error}")]
    TagDeserialization {
        path: PathBuf,
        tag: String,
        #[source]
        error: serde_json::Error,
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
