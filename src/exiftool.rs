use crate::error::ExifToolError;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;

use serde::de::DeserializeOwned;
use serde_json::Value;

const STDERR_POLL_INTERVAL: Duration = Duration::from_millis(5);
const STDERR_POLL_TIMEOUT: Duration = Duration::from_millis(50);

/// Main struct for interacting with the ExifTool process.
///
/// Maintains a persistent `exiftool` process in `-stay_open` mode for efficiency.
/// The process is automatically terminated when this struct is dropped.
///
/// **Note:** Most methods require `&mut self` because each command involves
/// stateful interaction with the underlying process (sending commands via stdin,
/// reading responses from stdout/stderr).
#[derive(Debug)]
pub struct ExifTool {
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    stderr_receiver: Receiver<String>,
    child: Child,
}

impl ExifTool {
    /// Launches the `exiftool` process in stay-open mode.
    ///
    /// Returns an error if the `exiftool` command cannot be found or started.
    pub fn new() -> Result<Self, ExifToolError> {
        Self::with_executable(Path::new("exiftool"))
    }

    /// Launches `exiftool` from a specific path.
    pub fn with_executable(exiftool_path: &Path) -> Result<Self, ExifToolError> {
        let mut child = Command::new(exiftool_path)
            .arg("-stay_open")
            .arg("True")
            .arg("-@")
            .arg("-") // Read command args from stdin
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(ExifToolError::ExifToolNotFound)?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| std::io::Error::other("Failed to capture stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| std::io::Error::other("Failed to capture stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| std::io::Error::other("Failed to capture stderr"))?;

        // Spawn a thread to continuously read stderr
        let (stderr_sender, stderr_receiver) = mpsc::channel();
        let stderr_reader = BufReader::new(stderr);
        thread::spawn(move || {
            // Send errors line by line. If the channel disconnects, the thread exits.
            for line in stderr_reader.lines().map_while(Result::ok) {
                if stderr_sender.send(line).is_err() {
                    // Receiver has been dropped, exiftool process likely closing
                    break;
                }
            }
            // Stderr stream closed or channel disconnected
        });

        Ok(Self {
            stdin: BufWriter::new(stdin),
            stdout: BufReader::new(stdout),
            stderr_receiver,
            child,
        })
    }

    // --- Core Execution Logic ---

    /// Executes a command with the provided arguments and returns the raw byte output.
    ///
    /// This is the low-level method used by other helpers. Users typically don't
    /// need to call this directly.
    ///
    /// # Arguments
    /// * `args` - A slice of string arguments to pass to `exiftool`.
    ///
    /// # Command Format Sent to ExifTool via Stdin
    /// ```text
    /// arg1
    /// arg2
    /// ...
    /// -execute
    /// ```
    pub fn execute_raw(&mut self, args: &[&str]) -> Result<Vec<u8>, ExifToolError> {
        // 1. Clear any stale errors from previous commands
        // This prevents misattribution of errors if a prior command failed silently
        // or if stderr wasn't fully drained.
        while self.stderr_receiver.try_recv().is_ok() {}

        // 2. Send command arguments line-by-line
        for arg in args {
            writeln!(self.stdin, "{}", arg)?;
        }
        // 3. Send the execute signal
        writeln!(self.stdin, "-execute")?;
        self.stdin.flush()?;

        // 4. Read the response from stdout
        let stdout_bytes = self.read_response_until_ready()?;
        // if !stdout_bytes.is_empty() {
        //     return Ok(stdout_bytes);
        // }

        // 5. Check for errors on stderr
        // ExifTool often prints errors *before* the "{ready}" signal for failed commands.
        let stderr_lines = self.drain_stderr()?;

        // 6. Process results and errors
        if !stderr_lines.is_empty() {
            // Combine args for error reporting
            let command_args = args.join(" ");
            let combined_stderr = stderr_lines.join("\n");

            // Check for specific common errors first
            for err_line in &stderr_lines {
                if let Some(filename) = err_line.strip_prefix("Error: File not found - ") {
                    return Err(ExifToolError::FileNotFound {
                        path: PathBuf::from(filename.trim()),
                        command_args,
                    });
                } else if err_line.contains("Error:") {
                    return Err(ExifToolError::ExifToolProcess {
                        message: err_line.to_string(),
                        std_err: combined_stderr,
                        command_args,
                    });
                } else if err_line.contains("Warning:") {
                    println!("ExifTool Warning - {}", err_line);
                }
            }
        }

        // If stderr was empty, return the stdout bytes
        Ok(stdout_bytes)
    }

    /// Reads from stdout until the `exiftool` "{ready}" marker is found.
    fn read_response_until_ready(&mut self) -> Result<Vec<u8>, ExifToolError> {
        let mut buffer = Vec::with_capacity(4096);
        let ready_marker_unix = b"{ready}\n";
        let ready_marker_win = b"{ready}\r\n";

        loop {
            let mut chunk = [0u8; 4096];
            let bytes_read = self.stdout.read(&mut chunk)?;

            if bytes_read == 0 {
                // EOF before "{ready}" means the process likely terminated.
                // Try draining stderr one last time to capture potential fatal errors.
                let stderr_lines = self.drain_stderr().unwrap_or_default();
                return if !stderr_lines.is_empty() {
                    Err(ExifToolError::ExifToolProcess {
                        std_err: stderr_lines.join("\n"),
                        message: format!(
                            "Process terminated unexpectedly. Stderr:\n{}",
                            stderr_lines.join("\n")
                        ),
                        command_args: "<unknown - process terminated>".to_string(),
                    })
                } else {
                    Err(ExifToolError::ProcessTerminated)
                };
            }

            buffer.extend_from_slice(&chunk[..bytes_read]);

            // Check if the buffer ends with either ready marker.
            // Check windows first as it's longer.
            if buffer.len() >= ready_marker_win.len() && buffer.ends_with(ready_marker_win) {
                buffer.truncate(buffer.len() - ready_marker_win.len());
                return Ok(buffer);
            }
            if buffer.len() >= ready_marker_unix.len() && buffer.ends_with(ready_marker_unix) {
                buffer.truncate(buffer.len() - ready_marker_unix.len());
                return Ok(buffer);
            }
        }
    }

    /// Drains the stderr channel, collecting recent error messages.
    ///
    /// Uses a short polling mechanism as stderr messages might arrive slightly
    /// after the stdout response. This is a pragmatic approach for a synchronous wrapper.
    fn drain_stderr(&mut self) -> Result<Vec<String>, ExifToolError> {
        let mut err_lines = Vec::new();
        let start_time = Instant::now();

        // First, quickly drain any immediately available messages
        loop {
            match self.stderr_receiver.try_recv() {
                Ok(line) => err_lines.push(line),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return Err(ExifToolError::StderrDisconnected),
            }
        }

        // Then, poll briefly for any messages that might be slightly delayed
        while start_time.elapsed() < STDERR_POLL_TIMEOUT {
            match self.stderr_receiver.try_recv() {
                Ok(line) => err_lines.push(line),
                Err(TryRecvError::Empty) => {
                    // If we already have errors, assume the burst is over.
                    // If not, sleep briefly and try again.
                    if !err_lines.is_empty() {
                        break;
                    }
                    thread::sleep(STDERR_POLL_INTERVAL);
                }
                Err(TryRecvError::Disconnected) => {
                    // If disconnect happens *while* polling, report it,
                    // but return any errors collected so far.
                    if err_lines.is_empty() {
                        return Err(ExifToolError::StderrDisconnected);
                    } else {
                        break; // Return collected lines below
                    }
                }
            }
        }

        Ok(err_lines)
    }

    /// Closes the persistent exiftool process gracefully.
    /// Called automatically when `ExifTool` is dropped.
    pub fn close(&mut self) -> Result<(), std::io::Error> {
        // Send the command to exit stay_open mode
        writeln!(self.stdin, "-stay_open")?;
        writeln!(self.stdin, "False")?;
        writeln!(self.stdin, "-execute")?;
        self.stdin.flush()?;

        Ok(())
    }

    // --- Public Helper Methods ---

    /// Executes a command and returns the output as lines of strings.
    ///
    /// Runs `exiftool {args...}`.
    ///
    /// # Example
    /// ```no_run
    /// # use exiftool::{ExifTool, ExifToolError};
    /// # use std::path::Path;
    /// # fn main() -> Result<(), ExifToolError> {
    /// let mut exiftool = ExifTool::new()?;
    /// let output_lines = exiftool.execute_lines(&["-S", "-DateTimeOriginal", "data/image.jpg"])?;
    /// for line in output_lines {
    ///     println!("{}", line);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn execute_lines(&mut self, args: &[&str]) -> Result<Vec<String>, ExifToolError> {
        let raw_output = self.execute_raw(args)?;
        let output_string = String::from_utf8(raw_output)?;
        Ok(output_string.lines().map(String::from).collect())
    }

    /// Executes a command with `-json` and returns the parsed `serde_json::Value`.
    ///
    /// Runs `exiftool -json {args...}`.
    /// ExifTool's JSON output is typically an array, even for a single file.
    ///
    /// # Example
    /// ```no_run
    /// # use exiftool::{ExifTool, ExifToolError};
    /// # use std::path::Path;
    /// # fn main() -> Result<(), ExifToolError> {
    /// let mut exiftool = ExifTool::new()?;
    /// let json_output = exiftool.json_execute(&["-g1", "-Author", "data/image.jpg", "another.png"])?;
    /// if let Some(array) = json_output.as_array() {
    ///     for item in array {
    ///         println!("Metadata: {}", item);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn json_execute(&mut self, args: &[&str]) -> Result<Value, ExifToolError> {
        let mut cmd_args = vec!["-json"];
        cmd_args.extend_from_slice(args);
        let output_bytes = self.execute_raw(&cmd_args)?;
        // Handle empty output gracefully - ExifTool might return empty output for
        // certain commands or errors that weren't caught via stderr.
        if output_bytes.is_empty() {
            // Or return Ok(Value::Null) or Ok(Value::Array(vec![])) ?
            return Err(ExifToolError::UnexpectedFormat {
                path: args
                    .iter()
                    .find(|a| !a.starts_with('-'))
                    .unwrap_or(&"<unknown>")
                    .to_string(),
                command_args: cmd_args.join(" "),
            });
        }
        let value: Value = serde_json::from_slice(&output_bytes)?;
        Ok(value)
    }

    // --- Reading Metadata ---

    /// Reads metadata for one or more files, returning raw JSON `Value`s.
    /// Adds `-json` automatically. Use `extra_args` for options like `-g1`, `-common`, etc.
    ///
    /// Runs `exiftool -json {extra_args...} {file_paths...}`.
    ///
    /// Returns a `Vec<Value>`, where each `Value` corresponds to a file.
    ///
    /// # Arguments
    /// * `file_paths`: An iterator of paths to process.
    /// * `extra_args`: Additional arguments like `-g1`, `-DateTimeFormat`, etc.
    ///
    /// # Example
    /// ```no_run
    /// # use exiftool::{ExifTool, ExifToolError};
    /// # use std::path::Path;
    /// # fn main() -> Result<(), ExifToolError> {
    /// let mut exiftool = ExifTool::new()?;
    /// let paths = [Path::new("image1.jpg"), Path::new("image2.png")];
    /// let results = exiftool.json_batch(paths, &["-g1", "-common"])?;
    /// assert_eq!(results.len(), 2);
    /// println!("Metadata for first file: {}", results[0]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn json_batch<I, P>(
        &mut self,
        file_paths: I,
        extra_args: &[&str],
    ) -> Result<Vec<Value>, ExifToolError>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let path_strs: Vec<String> = file_paths
            .into_iter()
            .map(|p| p.as_ref().to_string_lossy().into_owned())
            .collect();

        if path_strs.is_empty() {
            return Ok(Vec::new());
        }

        let mut args = extra_args.to_vec();
        // Convert path_strs to &str for the args slice
        let path_refs: Vec<&str> = path_strs.iter().map(String::as_str).collect();
        args.extend_from_slice(&path_refs);

        let result_value = self.json_execute(&args)?;

        match result_value {
            Value::Array(array) => Ok(array),
            _ => Err(ExifToolError::UnexpectedFormat {
                path: path_strs.join(", "),
                command_args: format!("-json {}", args.join(" ")),
            }),
        }
    }

    /// Reads metadata for a single file, returning a raw JSON `Value`.
    /// Adds `-json` automatically. Use `extra_args` for options like `-g1`, `-common`, etc.
    ///
    /// Runs `exiftool -json {extra_args...} {file_path}`.
    ///
    /// Extracts the single JSON object from the array ExifTool returns.
    /// Returns `TagNotFound` if ExifTool returns an empty array (e.g., file not found error handled internally).
    ///
    /// # Example
    /// ```no_run
    /// # use exiftool::{ExifTool, ExifToolError};
    /// # use std::path::Path;
    /// # fn main() -> Result<(), ExifToolError> {
    /// let mut exiftool = ExifTool::new()?;
    /// let path = Path::new("data/image.jpg");
    /// let result = exiftool.json(path, &["-g1", "-common"])?;
    /// println!("Metadata: {}", result);
    /// # Ok(())
    /// # }
    /// ```
    pub fn json(&mut self, file_path: &Path, extra_args: &[&str]) -> Result<Value, ExifToolError> {
        let path_str = file_path.to_string_lossy();
        let mut args = extra_args.to_vec();
        args.push(path_str.as_ref());

        let results = self.json_batch(std::iter::once(file_path), extra_args)?;

        results.into_iter().next().ok_or_else(|| {
            // This may happen if exiftool had an error (like file not found)
            // but it was suppressed or not parsed correctly from stderr.
            // We treat it as if the primary data wasn't found for the file.
            ExifToolError::UnexpectedFormat {
                path: file_path.to_string_lossy().into_owned(),
                command_args: args.join(" "),
            }
        })
    }

    /// Reads specific tags for a single file and deserializes into `T`.
    /// Adds `-json` automatically. Tags without values will be missing from the JSON.
    ///
    /// Runs `exiftool -json {-TAG...} {file_path}`.
    ///
    /// # Arguments
    /// * `file_path`: Path to the file.
    /// * `tags`: A slice of tag names (e.g., `"Author"`, `"ImageWidth"`). Do *not* include the leading `-`.
    ///
    /// # Example
    /// ```no_run
    /// # use exiftool::{ExifTool, ExifToolError};
    /// # use std::path::Path;
    /// # use serde::Deserialize;
    /// #[derive(Deserialize, Debug)]
    /// #[serde(rename_all = "PascalCase")]
    /// struct LensInfo { mega_pixels: Option<f64>, focal_length: Option<String> }
    ///
    /// # fn main() -> Result<(), ExifToolError> {
    /// let mut exiftool = ExifTool::new()?;
    /// let path = Path::new("photo.raw");
    /// let lens: LensInfo = exiftool.read_tags(path, &["LensID", "FocalLength"])?;
    /// println!("{:?}", lens);
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_tags<T: DeserializeOwned>(
        &mut self,
        file_path: &Path,
        tags: &[&str],
    ) -> Result<T, ExifToolError> {
        let tag_args: Vec<String> = tags.iter().map(|t| format!("-{}", t)).collect();
        let tag_args_str: Vec<&str> = tag_args.iter().map(String::as_str).collect();

        let value = self.json(file_path, &tag_args_str)?;

        // Use serde_path_to_error for better context on failure
        serde_path_to_error::deserialize(value).map_err(|e| ExifToolError::Deserialization {
            path: e.path().to_string(),
            source: e.into_inner(),
        })
    }

    /// Reads *all* metadata for a single file and deserializes into `T`.
    /// Adds `-json` automatically. Use `extra_args` for options like `-g1`, `-g2`, etc.
    ///
    /// Runs `exiftool -json {extra_args...} {file_path}`.
    ///
    /// # Example
    /// ```no_run
    /// # use exiftool::{ExifTool, ExifToolError, ExifData};
    /// # use std::path::Path;
    /// # fn main() -> Result<(), ExifToolError> {
    /// let mut exiftool = ExifTool::new()?;
    /// let path = Path::new("data/image.jpg");
    /// // Use -g1 for grouped output compatible with ExifData struct
    /// let metadata: ExifData = exiftool.read_metadata(path, &["-g2"])?;
    /// println!("Make: {:?}", metadata.camera.and_then(|e| e.make));
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_metadata<T: DeserializeOwned>(
        &mut self,
        file_path: &Path,
        extra_args: &[&str],
    ) -> Result<T, ExifToolError> {
        let value = self.json(file_path, extra_args)?;
        serde_path_to_error::deserialize(value).map_err(ExifToolError::from)
    }

    /// Reads a single tag's value as a JSON `Value`.
    ///
    /// Runs `exiftool -json -TAG {file_path}`.
    /// Returns `TagNotFound` if the tag doesn't exist in the file's metadata.
    pub fn json_tag(&mut self, file_path: &Path, tag: &str) -> Result<Value, ExifToolError> {
        let tag_arg = format!("-{}", tag);
        // Read *only* this tag using the metadata endpoint
        let metadata_json = self.json(file_path, &[&tag_arg])?;

        // The result is an object like {"SourceFile": "...", "TAG": ...}
        metadata_json
            .get(tag)
            .cloned()
            .ok_or_else(|| ExifToolError::TagNotFound {
                path: file_path.to_path_buf(),
                tag: tag.to_string(),
            })
    }

    /// Reads a single tag and deserializes its value into `T`.
    ///
    /// Runs `exiftool -json -TAG {file_path}` and extracts the tag's value.
    ///
    /// - If the tag exists and deserializes correctly into `T`, returns `Ok(T)`.
    /// - If the tag exists but its value cannot be deserialized into `T`, returns `Err(TagDeserialization)`.
    /// - If the tag does not exist *and* `T` is `Option<Inner>`, returns `Ok(None)`.
    /// - If the tag does not exist *and* `T` is *not* `Option<Inner>` (or something else that accepts `null`),
    ///   returns `Err(TagNotFound)`.
    /// - Propagates any other errors (IO, process errors) from accessing the tag.
    ///
    /// # Example
    /// ```no_run
    /// # use exiftool::{ExifTool, ExifToolError};
    /// # use std::path::Path;
    /// # fn main() -> Result<(), ExifToolError> {
    /// let mut exiftool = ExifTool::new()?;
    /// let path = Path::new("data/image.jpg");
    ///
    /// // T = String, tag exists -> Ok("Canon")
    /// let make: String = exiftool.read_tag(path, "Make")?;
    ///
    /// // T = u32, tag exists -> Ok(1234)
    /// let width: u32 = exiftool.read_tag(path, "ImageWidth")?;
    ///
    /// // T = Option<String>, tag exists -> Ok(Some("Description"))
    /// let desc: Option<String> = exiftool.read_tag(path, "ImageDescription")?;
    ///
    /// // T = Option<String>, tag *missing* -> Ok(None)
    /// let missing_opt: Option<String> = exiftool.read_tag(path, "NonExistentTag1")?;
    /// assert_eq!(missing_opt, None);
    ///
    /// // T = String, tag *missing* -> Err(TagNotFound)
    /// let missing_req: Result<String, ExifToolError> = exiftool.read_tag(path, "NonExistentTag2");
    /// assert!(matches!(missing_req, Err(ExifToolError::TagNotFound { .. })));
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_tag<T: DeserializeOwned>(
        &mut self,
        file_path: &Path,
        tag: &str,
    ) -> Result<T, ExifToolError> {
        // Step 1: Attempt to get the JSON value for the tag
        let value_result = self.json_tag(file_path, tag);

        match value_result {
            // Case 1: Tag found, value exists. Try to deserialize it directly.
            Ok(value) => {
                serde_json::from_value(value).map_err(|e| ExifToolError::TagDeserialization {
                    path: file_path.to_path_buf(),
                    tag: tag.to_string(),
                    error: e,
                })
            }

            // Case 2: Tag specifically not found by the underlying method.
            // Now we need to check if T expects an Option.
            Err(ExifToolError::TagNotFound { .. }) => {
                // Try to deserialize `Value::Null`. This only works if T can handle `null`
                // (most commonly, if T is Option<Inner>).
                match serde_json::from_value(Value::Null) {
                    // If deserializing `null` succeeds, it implies T is Option-like.
                    // The result `val` will be the `None` variant wrapped in T (which is Option<Inner>).
                    Ok(val) => Ok(val),

                    // If deserializing `null` fails, it means T was *not* expecting an Option
                    // (e.g., T is String, u32). In this situation, the original TagNotFound
                    // error is the correct one to surface.
                    Err(_) => Err(ExifToolError::TagNotFound {
                        // Reconstruct the specific error
                        path: file_path.to_path_buf(),
                        tag: tag.to_string(),
                    }),
                }
            }

            // Case 3: Any other error occurred while fetching the tag (IO, process error, etc.)
            Err(e) => Err(e), // Propagate the underlying error
        }
    }

    /// Reads a binary tag (like ThumbnailImage, PreviewImage) as raw bytes.
    ///
    /// Runs `exiftool -b -TAG {file_path}`.
    /// Returns `TagNotFound` if the tag doesn't exist or has no binary data.
    ///
    /// # Example
    /// ```
    /// # use exiftool::{ExifTool, ExifToolError};
    /// # use std::path::Path;
    /// # fn main() -> Result<(), ExifToolError> {
    /// let mut exiftool = ExifTool::new()?;
    /// let path = Path::new("data/image.jpg");
    /// let bytes = exiftool.read_tag_binary(path, "ThumbnailImage")?;
    /// dbg!(bytes);
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_tag_binary(
        &mut self,
        file_path: &Path,
        tag: &str,
    ) -> Result<Vec<u8>, ExifToolError> {
        let tag_arg = format!("-{}", tag);
        let path_str = file_path.to_string_lossy();
        let args = [path_str.as_ref(), "-b", &tag_arg];

        let bytes = self.execute_raw(&args)?;

        if bytes.is_empty() {
            // Assume empty binary output means tag not found for simplicity.
            return Err(ExifToolError::TagNotFound {
                path: file_path.to_path_buf(),
                tag: tag.to_string(),
            });
        }
        Ok(bytes)
    }

    // --- Writing Metadata ---

    /// Writes a value to a specific tag.
    ///
    /// Runs `exiftool {-TAG=VALUE} {extra_args...} {file_path}`.
    ///
    /// **Warning:** ExifTool will create a backup file when writing (`{filename}_original`).
    ///
    /// # Arguments
    /// * `file_path`: Path to the file to modify.
    /// * `tag`: The tag name (e.g., `"Author"`).
    /// * `value`: The value to write. It will be converted to a string.
    /// * `extra_args`: Args like `-overwrite_original`, `-P`.
    ///
    /// # Example
    /// ```no_run
    /// # use exiftool::{ExifTool, ExifToolError};
    /// # use std::path::Path;
    /// # fn main() -> Result<(), ExifToolError> {
    /// let mut exiftool = ExifTool::new()?;
    /// let path = Path::new("data/image.jpg");
    /// exiftool.write_tag(path, "UserComment", "My important comment", &[])?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_tag<T: ToString>(
        &mut self,
        file_path: &Path,
        tag: &str,
        value: T,
        extra_args: &[&str],
    ) -> Result<(), ExifToolError> {
        let value_str = value.to_string();
        let tag_arg = format!("-{}={}", tag, value_str);
        let path_str = file_path.to_string_lossy();

        let mut args = vec![tag_arg.as_str()];
        args.extend_from_slice(extra_args);
        args.push(path_str.as_ref());

        // Execute and ignore the output bytes (usually just "1 image files updated")
        let _ = self.execute_raw(&args)?;
        Ok(())
    }

    /// Writes raw binary data to a tag (e.g., ThumbnailImage).
    ///
    /// Uses a temporary file to pass the data to ExifTool.
    /// Runs `exiftool {-TAG<=TEMP_FILE} {extra_args...} {file_path}`.
    ///
    /// **Warning:** ExifTool will create a backup file when writing (`{filename}_original`).
    ///
    /// # Arguments
    /// * `file_path`: Path to the file to modify.
    /// * `tag`: The tag name (e.g., `"ThumbnailImage"`).
    /// * `data`: The binary data to write.
    /// * `extra_args`: Args like `-overwrite_original`, `-P`.
    ///
    /// # Example
    /// ```no_run
    /// # use exiftool::{ExifTool, ExifToolError};
    /// # use std::path::Path;
    /// # use std::fs;
    /// # fn main() -> Result<(), ExifToolError> {
    /// let mut exiftool = ExifTool::new()?;
    /// let path = Path::new("data/image.jpg");
    /// let new_thumbnail_bytes = fs::read("new_thumb.jpg").expect("Failed to read thumbnail");
    /// exiftool.write_tag_binary(path, "ThumbnailImage", &new_thumbnail_bytes, &[])?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_tag_binary<D: AsRef<[u8]>>(
        &mut self,
        file_path: &Path,
        tag: &str,
        data: D,
        extra_args: &[&str],
    ) -> Result<(), ExifToolError> {
        // Create a temporary file to hold the binary data
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(data.as_ref())?;
        temp_file.flush()?;

        let temp_path_str = temp_file.path().to_string_lossy();

        // Construct the field argument with the '<=' operator.
        let tag_arg = format!("-{}<={}", tag, temp_path_str);

        let file_path_str = file_path.to_string_lossy();
        let mut args = vec![tag_arg.as_str()];
        args.extend_from_slice(extra_args);
        args.push(file_path_str.as_ref());

        // Execute and ignore output. temp_file is dropped (and deleted) after this scope.
        let _ = self.execute_raw(&args)?;
        Ok(())
    }
}

impl Drop for ExifTool {
    /// Attempts to gracefully close the `exiftool` process and then kills it
    /// if it hasn't terminated.
    fn drop(&mut self) {
        // Attempt graceful shutdown first. Ignore errors, as we'll kill anyway.
        let _ = self.close();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::test_helpers::{list_files_recursive, test_image_path}; // Use updated helper
    use assert_matches::assert_matches;
    use image::ImageReader;
    use serde::Deserialize;
    use serde_json::json;
    use std::fs;
    use std::io::Cursor;

    // Helper to create a temporary copy of the test image
    fn setup_temp_image() -> Result<PathBuf, std::io::Error> {
        let src_path = test_image_path();
        let (_, pb) = tempfile::Builder::new()
            .suffix(".jpg")
            .tempfile_in("data")?
            .keep()?;
        fs::copy(&src_path, &pb)?;
        Ok(pb)
    }

    #[test]
    fn test_new_ok() {
        assert!(ExifTool::new().is_ok());
    }

    #[test]
    fn test_new_invalid_path() {
        let result = ExifTool::with_executable(Path::new("nonexistent_exiftool_command"));
        assert_matches!(result, Err(ExifToolError::ExifToolNotFound(_)));
    }

    #[test]
    fn test_execute_lines_ok() -> Result<(), ExifToolError> {
        let mut et = ExifTool::new()?;
        let path = test_image_path();
        let lines = et.execute_lines(&["-S", "-FocalLength", path.to_str().unwrap()])?;
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("FocalLength: 4.7 mm"));
        Ok(())
    }

    #[test]
    fn test_file_not_found_error() -> Result<(), ExifToolError> {
        let mut et = ExifTool::new()?;
        let non_existent_path = Path::new("data/non_existent_file.jpg");
        let result = et.json(non_existent_path, &[]);
        assert_matches!(
            result,
            Err(ExifToolError::FileNotFound { path, .. } ) if path == non_existent_path
        );

        // Test raw execution too
        let raw_result = et.execute_raw(&[non_existent_path.to_str().unwrap()]);
        assert_matches!(
            raw_result,
            Err(ExifToolError::FileNotFound { path, .. }) if path == non_existent_path
        );
        Ok(())
    }

    #[test]
    fn test_read_metadata_json_single() -> Result<(), ExifToolError> {
        let mut et = ExifTool::new()?;
        let path = test_image_path();
        let meta = et.json(path.as_path(), &["-Make", "-Model"])?;

        assert!(meta.is_object());
        assert_eq!(
            meta.get("SourceFile").and_then(|v| v.as_str()),
            Some(path.to_str().unwrap())
        );
        assert_eq!(meta.get("Make").and_then(|v| v.as_str()), Some("Huawei"));
        assert_eq!(meta.get("Model").and_then(|v| v.as_str()), Some("Nexus 6P"));
        Ok(())
    }

    #[test]
    fn test_read_metadata_json_batch() -> Result<(), ExifToolError> {
        let mut et = ExifTool::new()?;
        let path1 = test_image_path();
        let path2 = PathBuf::from("data/valid/other_images/jpg/gps/DSCN0010.jpg");
        let paths = vec![path1.as_path(), path2.as_path()];
        let meta_list = et.json_batch(paths, &["-FileName", "-FileSize"])?;

        assert_eq!(meta_list.len(), 2);
        assert!(meta_list[0].is_object());
        assert!(meta_list[1].is_object());
        assert_eq!(
            meta_list[0].get("FileName").and_then(Value::as_str),
            Some(path1.file_name().unwrap().to_str().unwrap())
        );
        assert_eq!(
            meta_list[1].get("FileName").and_then(Value::as_str),
            Some(path2.file_name().unwrap().to_str().unwrap())
        );
        Ok(())
    }

    #[test]
    fn test_read_tag_json() -> Result<(), ExifToolError> {
        let mut et = ExifTool::new()?;
        let path = test_image_path();
        let make = et.json_tag(path.as_path(), "Make")?;
        assert_eq!(make, json!("Huawei"));
        Ok(())
    }

    #[test]
    fn test_read_tag_json_not_found() -> Result<(), ExifToolError> {
        let mut et = ExifTool::new()?;
        let path = test_image_path();
        let result = et.json_tag(path.as_path(), "NonExistentTag123");
        assert_matches!(
            result,
            Err(ExifToolError::TagNotFound { tag, .. }) if tag == "NonExistentTag123"
        );
        Ok(())
    }

    #[test]
    fn test_read_tag_generic() -> Result<(), ExifToolError> {
        let mut et = ExifTool::new()?;
        let path = test_image_path();

        let make: String = et.read_tag(path.as_path(), "Make")?;
        assert_eq!(make, "Huawei");

        let width: u32 = et.read_tag(path.as_path(), "ImageWidth")?;
        assert_eq!(width, 2688);

        // Test Option for present tag
        let desc: Option<String> = et.read_tag(path.as_path(), "Model")?;
        assert!(desc.is_some());

        // Test Option for missing tag
        let desc: Option<String> = et.read_tag(path.as_path(), "ImageDescription")?;
        assert!(desc.is_none());

        let missing: Result<String, _> = et.read_tag(path.as_path(), "NonExistentTag456");
        assert_matches!(
            missing,
            Err(ExifToolError::TagNotFound { tag, .. }) if tag == "NonExistentTag456"
        );

        // Test deserialization failure
        let width_as_string: Result<String, _> = et.read_tag(path.as_path(), "ImageWidth");
        assert_matches!(
            width_as_string,
            Err(ExifToolError::TagDeserialization{ tag, .. }) if tag == "ImageWidth"
        );

        Ok(())
    }

    #[test]
    fn test_read_tags_struct() -> Result<(), ExifToolError> {
        #[derive(Deserialize, Debug, PartialEq)]
        #[serde(rename_all = "PascalCase")] // Match ExifTool's typical tag names
        struct CameraInfo {
            make: String,
            model: String,
            image_width: u32,
            software: Option<String>, // Handle optional tags
        }

        let mut et = ExifTool::new()?;
        let path = test_image_path();
        let info: CameraInfo =
            et.read_tags(path.as_path(), &["Make", "Model", "ImageWidth", "Software"])?;

        assert_eq!(info.make, "Huawei");
        assert_eq!(info.model, "Nexus 6P");
        assert!(info.image_width > 0);
        assert!(info.software.is_some());
        Ok(())
    }

    #[test]
    fn test_read_tag_binary() -> Result<(), ExifToolError> {
        let mut et = ExifTool::new()?;
        let path = test_image_path();
        let thumb_bytes = et.read_tag_binary(path.as_path(), "ThumbnailImage")?;
        assert!(!thumb_bytes.is_empty());
        // Add basic JPEG check if needed (requires image crate)
        assert!(thumb_bytes.starts_with(b"\xFF\xD8")); // JPEG SOI marker
        assert!(thumb_bytes.ends_with(b"\xFF\xD9")); // JPEG EOI marker
        Ok(())
    }

    #[test]
    fn test_read_tag_binary_image() -> Result<(), ExifToolError> {
        let mut et = ExifTool::new()?;
        let path = test_image_path();
        let thumb_bytes = et.read_tag_binary(path.as_path(), "ThumbnailImage")?;

        dbg!(thumb_bytes.len());
        // Verify it's a valid JPEG
        let cursor = Cursor::new(&thumb_bytes);
        let format = ImageReader::new(cursor)
            .with_guessed_format()
            .expect("Cursor never fails")
            .format();

        assert_eq!(format, Some(image::ImageFormat::Jpeg));

        // decode to check that it's readable
        let img = image::load_from_memory(&thumb_bytes).unwrap();
        println!("Thumbnail dimensions: {}x{}", img.width(), img.height());

        Ok(())
    }

    #[test]
    fn test_read_tag_binary_not_found() -> Result<(), ExifToolError> {
        let mut et = ExifTool::new()?;
        let path = test_image_path();
        let result = et.read_tag_binary(path.as_path(), "NonExistentBinaryTag");
        assert_matches!(
            result,
            Err(ExifToolError::TagNotFound { tag, .. }) if tag == "NonExistentBinaryTag"
        );
        Ok(())
    }

    #[test]
    fn test_write_tag_string() -> Result<(), ExifToolError> {
        let mut et = ExifTool::new()?;
        let temp_img = setup_temp_image()?;

        // Write string
        let new_author = "Rust Writer Test";
        et.write_tag(&temp_img, "Author", new_author, &[])?;

        let read_author: String = et.read_tag(&temp_img, "Author")?;
        assert_eq!(read_author, new_author);

        // Write integer
        let new_iso = 2897;
        et.write_tag(&temp_img, "ISO", new_iso, &[])?;

        let read_iso: u32 = et.read_tag(&temp_img, "ISO")?;
        assert_eq!(read_iso, new_iso);

        // Clean up
        fs::remove_file(&temp_img)?;
        fs::remove_file(format!("{}_original", &temp_img.display()))?;

        Ok(())
    }

    #[test]
    fn test_write_tag_binary() -> Result<(), ExifToolError> {
        let mut et = ExifTool::new()?;
        let temp_img = setup_temp_image()?;

        let dummy_thumb = b"\xFF\xD8\xFF\xE0\x00\x10JFIF\x00\x01\x01\x00\x00\x01\x00\x01\x00\x00\xFF\xDB\x00C\x00\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\xFF\xC0\x00\x11\x08\x00\x01\x00\x01\x03\x01\x22\x00\x02\x11\x01\x03\x11\x01\xFF\xC4\x00\x15\x00\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xFF\xDA\x00\x0C\x03\x01\x00\x02\x11\x03\x11\x00\x3F\x00\xA8\xFF\xD9"; // Tiny valid JPEG

        let j = et.write_tag_binary(&temp_img, "ThumbnailImage", dummy_thumb, &[]);
        assert!(j.is_ok());

        let read_thumb = et.read_tag_binary(&temp_img, "ThumbnailImage")?;
        fs::remove_file(&temp_img)?;
        fs::remove_file(format!("{}_original", &temp_img.display()))?;
        assert_eq!(read_thumb, dummy_thumb);

        Ok(())
    }

    #[test]
    fn test_read_metadata_full_struct() -> Result<(), ExifToolError> {
        use crate::ExifData;

        let mut et = ExifTool::new()?;
        let path = test_image_path();
        // Use the args required by the ExifData struct
        let metadata: ExifData = et.read_metadata(path.as_path(), &["-g2"])?;

        assert!(metadata.camera.is_some());
        assert_eq!(metadata.camera.unwrap().make.unwrap(), "Huawei");

        assert!(metadata.other.is_some());
        assert_eq!(
            metadata.other.unwrap().file_name.unwrap(),
            path.file_name().unwrap().to_str().unwrap()
        );

        Ok(())
    }

    #[test]
    fn test_batch_processing_robustness() -> Result<(), ExifToolError> {
        let test_dir = Path::new("data/valid");
        let files = list_files_recursive(test_dir).expect("Failed to list test files");
        assert!(!files.is_empty(), "No test files found in data/valid");

        let mut exiftool = ExifTool::new()?;
        // Use AsRef<Path> directly
        let results = exiftool.json_batch(files.iter(), &["-SourceFile"])?;

        assert_eq!(results.len(), files.len());

        for (i, result_val) in results.iter().enumerate() {
            let source_file = result_val
                .get("SourceFile")
                .and_then(Value::as_str)
                .map(PathBuf::from);
            assert!(
                source_file.is_some(),
                "SourceFile missing in result for index {}",
                i
            );
            // Note: Comparing paths directly can be tricky due to CWD differences.
            // Compare basenames or canonicalize if needed, but checking existence is good.
            assert!(
                source_file
                    .unwrap()
                    .ends_with(files[i].file_name().unwrap()),
                "Mismatch for file {}",
                files[i].display()
            );
        }
        Ok(())
    }
}
