use crate::error::ExifToolError;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

pub struct ExifTool {
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    error_receiver: Receiver<String>,
    child: Child,
}

impl ExifTool {
    /// Create an instance of ExifTool. The process will stay open until the instance is dropped.
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

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| std::io::Error::other("Failed to capture stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| std::io::Error::other("Failed to capture stdout"))?;
        // Capture stderr only once.
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| std::io::Error::other("Failed to capture stdin"))?;

        // Create a channel and spawn a thread to read stderr continuously.
        let (error_sender, error_receiver): (Sender<String>, Receiver<String>) = mpsc::channel();
        let stderr_reader = BufReader::new(stderr);
        thread::spawn(move || {
            for line in stderr_reader.lines().map_while(Result::ok) {
                let _ = error_sender.send(line);
            }
        });

        Ok(Self {
            stdin: BufWriter::new(stdin),
            stdout: BufReader::new(stdout),
            error_receiver,
            child,
        })
    }

    fn read_response(&mut self) -> Result<Vec<u8>, ExifToolError> {
        let mut buffer = Vec::new();
        let ready_marker_unix = b"{ready}\n"; // 7 bytes
        let ready_marker_win = b"{ready}\r\n"; // 8 bytes

        loop {
            let mut chunk = [0u8; 4096];
            let bytes_read = self.stdout.read(&mut chunk)?;

            if bytes_read == 0 {
                return Err(ExifToolError::ProcessTerminated);
            }

            buffer.extend_from_slice(&chunk[..bytes_read]);

            // Check for Windows marker first (longer)
            if buffer.len() >= ready_marker_win.len() {
                let win_start = buffer.len() - ready_marker_win.len();
                if &buffer[win_start..] == ready_marker_win {
                    buffer.truncate(win_start);
                    return Ok(buffer);
                }
            }

            // Check for Unix marker
            if buffer.len() >= ready_marker_unix.len() {
                let unix_start = buffer.len() - ready_marker_unix.len();
                if &buffer[unix_start..] == ready_marker_unix {
                    buffer.truncate(unix_start);
                    return Ok(buffer);
                }
            }
        }
    }

    fn get_error_lines(&mut self) -> Result<Vec<String>, ExifToolError> {
        // Give error messages a chance to arrive.
        let poll_timeout = Duration::from_millis(10);
        let poll_interval = Duration::from_millis(2);
        let start = Instant::now();
        let mut err_lines = Vec::new();

        while start.elapsed() < poll_timeout {
            loop {
                match self.error_receiver.try_recv() {
                    Ok(line) => err_lines.push(line),
                    Err(mpsc::TryRecvError::Empty) => break,
                    Err(mpsc::TryRecvError::Disconnected) => {
                        return Err(ExifToolError::ChannelDisconnected)
                    }
                }
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
    /// The command executed by this function is as follows:
    ///
    /// `exiftool {...args}`
    pub fn execute_raw(&mut self, cmd_args: &[&str]) -> Result<Vec<u8>, ExifToolError> {
        // Clear previous errors
        let _: Vec<String> = self.error_receiver.try_iter().collect();

        // Send command to exiftool.
        for arg in cmd_args {
            writeln!(self.stdin, "{}", arg)?;
        }
        writeln!(self.stdin, "-execute")?;
        self.stdin.flush()?;

        // Read stdout response until the ready marker.
        let stdout_bytes = self.read_response()?;
        if !stdout_bytes.is_empty() {
            return Ok(stdout_bytes);
        }
        let err_lines = self.get_error_lines()?;

        for err_line in &err_lines {
            if let Some(filename) = err_line.strip_prefix("Error: File not found - ") {
                return Err(ExifToolError::FileNotFound {
                    file: filename.trim().to_string(),
                    command: cmd_args.join(" "),
                });
            } else if err_line.contains("Error:") {
                return Err(ExifToolError::ExifToolError {
                    message: err_lines.join(" "),
                    command: cmd_args.join(" "),
                });
            }
        }
        Ok(stdout_bytes)
    }

    pub fn close(&mut self) -> Result<(), ExifToolError> {
        writeln!(self.stdin, "-stay_open")?;
        writeln!(self.stdin, "False")?;
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

    #[test]
    fn test_file_not_found() -> Result<(), ExifToolError> {
        let filename = "nonexistent.jpg";
        let mut exiftool = ExifTool::new()?;
        let result = exiftool.execute_raw(&[filename]);
        assert!(result.is_err());

        match result {
            Err(ExifToolError::FileNotFound {
                file: f,
                command: _,
            }) => {
                assert_eq!(f, filename);
                Ok(())
            }
            other => panic!("Expected FileNotFound error, got {:?}", other),
        }
    }
}
