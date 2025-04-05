use crate::error::ExifToolError;
use serde_json::Value;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Child, ChildStdin, ChildStdout};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

pub struct ExifTool {
    timeout: Duration,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    // Shared buffer to collect stderr output.
    error_receiver: Receiver<String>,
    child: Child,
}

impl ExifTool {
    pub fn new() -> Result<Self, ExifToolError> {
        Self::new_with_timeout(Duration::from_secs(5))
    }

    pub fn new_with_timeout(timeout: Duration) -> Result<Self, ExifToolError> {
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
        // Capture stderr only once.
        let stderr = child.stderr.take().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to capture stderr")
        })?;

        // Create a channel and spawn a thread to read stderr continuously.
        let (error_sender, error_receiver): (Sender<String>, Receiver<String>) = mpsc::channel();
        let stderr_reader = BufReader::new(stderr);
        thread::spawn(move || {
            for line in stderr_reader.lines() {
                if let Ok(l) = line {
                    let _ = error_sender.send(l);
                }
            }
        });

        Ok(Self {
            timeout,
            stdin: BufWriter::new(stdin),
            stdout: BufReader::new(stdout),
            error_receiver,
            child,
        })
    }

    fn read_response(&mut self) -> Result<Vec<u8>, ExifToolError> {
        // todo de timeout doet niks als er geen message meer komt en hij nog wel wacht
        let start = Instant::now();
        let mut output = Vec::new();
        let mut line = String::new();
        let ready_marker_windows = "{ready}\r\n";
        let ready_marker_unix = "{ready}\n";

        while start.elapsed() < self.timeout {
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

        if start.elapsed() >= self.timeout {
            return Err(ExifToolError::Timeout);
        }

        Ok(output)
    }

    fn get_error_lines(&mut self) -> Result<Vec<String>, ExifToolError> {
        // Give error messages a chance to arrive.
        let poll_timeout = Duration::from_millis(100);
        let poll_interval = Duration::from_millis(5);
        let start = Instant::now();
        let mut err_lines = Vec::new();

        while start.elapsed() < poll_timeout {
            while let Ok(err_line) = self.error_receiver.try_recv() {
                err_lines.push(err_line);
            }
            if !err_lines.is_empty() {
                // expect all errors to come in a burst
                break;
            }
            thread::sleep(poll_interval);
        }
        Ok(err_lines)
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

        if stdout_bytes.len() == 0 {
            let err_lines = self.get_error_lines()?;
            if err_lines.is_empty() {
                return Err(ExifToolError::EmptyResponse);
            }
            for err_line in &err_lines {
                if let Some(filename) = err_line.strip_prefix("Error: File not found - ") {
                    return Err(ExifToolError::FileNotFound(filename.trim().to_string()));
                }
            }
            return Err(ExifToolError::ExifTool(err_lines.join("\n")));
        }

        Ok(stdout_bytes)
    }

    pub fn execute_json(&mut self, args: &[&str]) -> Result<Value, ExifToolError> {
        let mut cmd_args = vec!["-json"];
        cmd_args.extend_from_slice(args);

        let output_bytes = self.execute_bytes(&cmd_args)?;
        let output = String::from_utf8(output_bytes)?;
        let value: Value = serde_json::from_str(&output)?;
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
        // todo this test doesnt always succeed (race condition or something? the resulting output is empty then)
        let filename = "nonexistent.jpg";
        let mut exiftool = ExifTool::new()?;
        let result = exiftool.execute_bytes(&[filename]);
        assert!(!result.is_ok());

        match result {
            Err(ExifToolError::FileNotFound(f)) => {
                assert_eq!(f, filename);
                Ok(())
            }
            other => panic!("Expected FileNotFound error, got {:?}", other),
        }
    }
}
