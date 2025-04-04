use serde_json::Value;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Child, ChildStdin, ChildStdout};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
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
    #[error("Operation timed out")]
    Timeout,
}

pub struct ExifTool {
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    // Shared buffer to collect stderr output.
    error_buffer: Arc<Mutex<Vec<String>>>,
    // Handle for the thread that reads stderr.
    _stderr_handle: std::thread::JoinHandle<()>,
    child: Child,
}

impl ExifTool {
    pub fn new() -> Result<Self, ExifToolError> {
        let mut child = std::process::Command::new("exiftool")
            .arg("-stay_open")
            .arg("True")
            .arg("-@")
            .arg("-")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to capture stdin")
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to capture stdout")
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to capture stderr")
        })?;

        // Create a shared buffer and spawn a thread to read stderr continuously.
        let error_buffer = Arc::new(Mutex::new(Vec::new()));
        let error_buffer_clone = Arc::clone(&error_buffer);
        let stderr_reader = BufReader::new(stderr);
        let _stderr_handle = std::thread::spawn(move || {
            for line in stderr_reader.lines() {
                if let Ok(l) = line {
                    let mut errors = error_buffer_clone.lock().unwrap();
                    errors.push(l);
                }
            }
        });

        Ok(Self {
            stdin: BufWriter::new(stdin),
            stdout: BufReader::new(stdout),
            error_buffer,
            _stderr_handle,
            child,
        })
    }

    fn read_response(&mut self) -> Result<Vec<u8>, ExifToolError> {
        let timeout = Duration::from_secs(5);
        let start = Instant::now();
        let mut output = Vec::new();
        let mut line = String::new();
        let ready_marker_windows = "{ready}\r\n";
        let ready_marker_unix = "{ready}\n";

        while start.elapsed() < timeout {
            line.clear();
            match self.stdout.read_line(&mut line) {
                Ok(0) => return Err(ExifToolError::ProcessTerminated),
                Ok(_) => {
                    if line == ready_marker_unix || line == ready_marker_windows {
                        break;
                    }
                    output.extend_from_slice(line.as_bytes());
                }
                Err(e) => return Err(e.into()),
            }
        }

        if start.elapsed() >= timeout {
            return Err(ExifToolError::Timeout);
        }

        Ok(output)
    }

    /// Executes the given command arguments and returns the raw response bytes.
    /// After reading stdout, it checks the shared error buffer for any stderr output.
    pub fn execute_bytes(&mut self, cmd_args: &[&str]) -> Result<Vec<u8>, ExifToolError> {
        // Send command to exiftool.
        for arg in cmd_args {
            writeln!(self.stdin, "{}", arg)?;
        }
        writeln!(self.stdin, "-execute")?;
        self.stdin.flush()?;

        // Read stdout response until the ready marker.
        let stdout_bytes = self.read_response()?;

        // Check for any stderr output collected by the background thread.
        let errors = self.error_buffer.lock().unwrap();
        if !errors.is_empty() {
            let err_str = errors.join("\n");
            // Check for "File not found" error pattern
            if let Some(filename) = err_str.strip_prefix("Error: File not found - ") {
                return Err(ExifToolError::FileNotFound(filename.trim().to_string()));
            }
            return Err(ExifToolError::ExifTool(err_str));
        }

        Ok(stdout_bytes)
    }

    pub fn execute_json(&mut self, args: &[&str]) -> Result<Value, ExifToolError> {
        let mut cmd_args = vec!["-json"];
        cmd_args.extend_from_slice(args);

        let output_bytes = self.execute_bytes(&cmd_args)?;
        let output = String::from_utf8(output_bytes)?;
        let value: Value = serde_json::from_str(&output)?;

        // Check for ExifTool errors in the JSON output.
        if let Value::Array(arr) = &value {
            for obj in arr {
                if let Value::Object(map) = obj {
                    if let Some(Value::String(err_msg)) = map.get("Error") {
                        if err_msg.contains("File not found") {
                            let filename = args.first().cloned().unwrap_or("<unknown>").to_string();
                            return Err(ExifToolError::FileNotFound(filename));
                        }
                        return Err(ExifToolError::ExifTool(err_msg.clone()));
                    }
                }
            }
        }

        Ok(value)
    }

    pub fn close(&mut self) -> Result<(), ExifToolError> {
        writeln!(self.stdin, "-stay_open\nFalse\n")?;
        self.stdin.flush()?;
        Ok(())
    }
}

impl Drop for ExifTool {
    fn drop(&mut self) {
        let _ = self.close();
        let _ = self.child.kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_basic_functionality() -> Result<(), ExifToolError> {
        let mut exiftool = ExifTool::new()?;
        let file = "test_data/IMG_20170801_162043.jpg";

        assert!(Path::new(file).exists(), "Test file doesn't exist");

        // First query
        let result = exiftool.execute_json(&[file])?;
        assert!(result.is_array());
        println!("First result: {:#?}", result);

        // Second query with same process
        let result2 = exiftool.execute_json(&["-createdate", file])?;
        assert!(result2.is_array());
        println!("Second result: {:#?}", result2);
        Ok(())
    }

    #[test]
    fn test_file_not_found() -> Result<(), ExifToolError> {
        let filename = "nonexistent.jpg";
        let mut exiftool = ExifTool::new()?;
        let result = exiftool.execute_json(&[filename]);

        match result {
            Err(ExifToolError::FileNotFound(f)) => {
                assert_eq!(f, filename);
                Ok(())
            }
            other => panic!("Expected FileNotFound error, got {:?}", other),
        }
    }
}
