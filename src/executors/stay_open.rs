use crate::error::ExifToolError;
use serde_json::Value;
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
        let poll_timeout = Duration::from_millis(100);
        let poll_interval = Duration::from_millis(5);
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

        if stdout_bytes.is_empty() {
            let err_lines = self.get_error_lines()?;
            if err_lines.is_empty() {
                return Err(ExifToolError::EmptyResponse);
            }
            for err_line in &err_lines {
                if let Some(filename) = err_line.strip_prefix("Error: File not found - ") {
                    return Err(ExifToolError::FileNotFound(filename.trim().to_string()));
                }
            }
            return Err(ExifToolError::ExifToolError(err_lines.join("\n")));
        }

        Ok(stdout_bytes)
    }

    /// Execute any command and get the result in JSON form.
    /// The command executed by this function is as follows:
    ///
    /// `exiftool -json {...args}`
    ///
    /// The output will be a json array of objects, each object describing one input file.
    /// You can pass as many files as you want to this.
    ///
    /// For example:
    /// ```rs
    /// let files = vec![file1, file2, file3];
    /// let value = execute_json(files)?;
    /// ```
    ///
    /// You can tell exiftool to structure the output by grouping into categories with `-g1` or `-g2`.
    pub fn execute_json(&mut self, args: &[&str]) -> Result<Value, ExifToolError> {
        let mut cmd_args = vec!["-json"];
        cmd_args.extend_from_slice(args);

        let output_bytes = self.execute_raw(&cmd_args)?;
        let output = String::from_utf8(output_bytes)?;
        let value: Value = serde_json::from_str(&output)?;
        Ok(value)
    }

    /// Extract bytes from a binary field.
    /// The command executed by this function is as follows:
    ///
    /// `exiftool {file_path} -b -{field_name}`
    pub fn binary_field(
        &mut self,
        file_path: &str,
        field_name: &str,
    ) -> Result<Vec<u8>, ExifToolError> {
        self.execute_raw(&vec![file_path, "-b", &format!("-{}", field_name)])
    }

    /// Get JSON metadata for a single file. This will return a single json object.
    /// The command executed by this function is as follows:
    ///
    /// `exiftool -json {file_path} {...extra_args}`
    ///
    /// You can tell exiftool to structure the output by grouping into categories with `-g1` or `-g2`.
    pub fn file_metadata(
        &mut self,
        file_path: &str,
        extra_args: &[&str],
    ) -> Result<Value, ExifToolError> {
        let mut args = vec![file_path];
        args.extend_from_slice(extra_args);
        let result = self.execute_json(&args)?;
        if let Some(single) = result.as_array().and_then(|a| a.get(0)) {
            Ok(single.clone())
        } else {
            Err(ExifToolError::UnexpectedFormat)
        }
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
    use crate::utils::test_helpers::list_files_recursive;
    use image::ImageReader;
    use std::io::Cursor;
    use std::path::Path;

    #[test]
    fn test_basic_functionality() -> Result<(), ExifToolError> {
        let mut exiftool = ExifTool::new()?;
        let file = "test_data/IMG_20170801_162043.jpg";

        assert!(Path::new(file).exists(), "Test file doesn't exist");

        // First query
        let result = exiftool.file_metadata(file, &[])?;
        assert!(result.is_object());
        println!("First result: {:#?}", result);

        // Second query with same process
        let result2 = exiftool.file_metadata(file, &["-createdate"])?;
        assert!(result2.is_object());
        println!("Second result: {:#?}", result2);
        Ok(())
    }

    #[test]
    fn test_file_not_found() -> Result<(), ExifToolError> {
        let filename = "nonexistent.jpg";
        let mut exiftool = ExifTool::new()?;
        let result = exiftool.execute_raw(&[filename]);
        assert!(result.is_err());

        match result {
            Err(ExifToolError::FileNotFound(f)) => {
                assert_eq!(f, filename);
                Ok(())
            }
            other => panic!("Expected FileNotFound error, got {:?}", other),
        }
    }

    #[test]
    fn test_binary_response() -> Result<(), ExifToolError> {
        let mut exiftool = ExifTool::new()?;
        let file = "test_data/IMG_20170801_162043.jpg";
        let result = exiftool.binary_field(file, "ThumbnailImage");

        match result {
            Ok(data) => {
                dbg!(data.len());
                // Verify it's a valid JPEG
                let cursor = Cursor::new(&data);
                let format = ImageReader::new(cursor)
                    .with_guessed_format()
                    .expect("Cursor never fails")
                    .format();

                assert_eq!(format, Some(image::ImageFormat::Jpeg));

                // decode to check that it's readable
                let img = image::load_from_memory(&data).unwrap();
                println!("Thumbnail dimensions: {}x{}", img.width(), img.height());

                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    #[test]
    fn test_all_exif_files() -> Result<(), ExifToolError> {
        let test_dir = "test_data/exiftool_images";

        // Collect all files in directory (non-recursive)
        let files = list_files_recursive(test_dir.as_ref())?;
        assert!(!files.is_empty());

        let mut exiftool = ExifTool::new()?;

        for file in files {
            let file_path = file.to_string_lossy();
            println!("\nTesting: {}", file_path);

            // Single full metadata extraction per file
            let result = exiftool.file_metadata(&file_path, &[])?;

            // Basic validation
            assert!(
                result.is_object(),
                "Expected JSON array for file {}",
                file_path
            );
            assert!(
                !result.as_object().unwrap().is_empty(),
                "Empty result for file {}",
                file_path
            );

            println!("Metadata for {}: {:#?}", file_path, result);
        }

        Ok(())
    }
}
