use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExifToolError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("File not found: {file}. command={command}")]
    FileNotFound { file: String, command: String },

    #[error("ExifTool error: {message}. command={command}")]
    ExifToolError { message: String, command: String },

    #[error("Process terminated unexpectedly.")]
    ProcessTerminated,

    #[error("stderr channel for exiftool disconnected.")]
    ChannelDisconnected,

    #[error("Operation timed out. command={command}")]
    Timeout { command: String },

    #[error("Expected different format from exiftool. file={file}, args={args}")]
    UnexpectedFormat { file: String, args: String },

    #[error("The required field does not exist. file={file}, field={field}")]
    FieldDoesNotExist { file: String, field: String },

    #[error("The required field does not exist. file={file}, field={field}")]
    FieldToStringFailed { file: String, field: String },

    #[error("Deserialization error at path '{path}': {source}")]
    Deserialization {
        path: String,
        source: serde_json::Error,
    },

    #[error("Deserialization error at for field '{field}', file={file}, error={error}")]
    FieldDeserializationError {
        field: String,
        file: String,
        error: String,
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
