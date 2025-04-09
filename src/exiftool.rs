use crate::error::ExifToolError;
use log::warn;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;

use serde::de::DeserializeOwned;
use serde_json::Value;

// std err can come in a tiny bit delayed after stdout, in which case we have to wait a
// millisecond or 2 to be able to read errors and warnings.
const STDERR_POLL_INTERVAL: Duration = Duration::from_millis(1);
const STDERR_POLL_TIMEOUT: Duration = Duration::from_millis(2);

/// Interacts with a persistent `exiftool` command-line process.
///
/// This struct manages the lifecycle of an `exiftool` instance running in `-stay_open` mode,
/// allowing for efficient execution of multiple commands without the overhead of starting
/// a new process each time.
///
/// Communication happens via the process's standard input, output, and error streams.
/// An internal thread monitors stderr for errors and warnings.
///
/// Most methods require `&mut self` because each command involves stateful interaction
/// with the underlying process (sending commands via stdin, reading responses from stdout/stderr).
///
/// The `exiftool` process is automatically terminated when this struct is dropped,
/// attempting a graceful shutdown first.
///
/// # Examples
///
/// ```no_run
/// use exiftool::{ExifTool, ExifToolError};
/// use std::path::Path;
///
/// fn main() -> Result<(), ExifToolError> {
///     // Create an ExifTool instance (launches the process)
///     let mut et = ExifTool::new()?;
///
///     // Use methods to interact with exiftool...
///     let path = Path::new("image.jpg");
///     let width: u32 = et.read_tag(path, "ImageWidth")?;
///     println!("Width: {}", width);
///
///     // The process is automatically closed when `et` goes out of scope
///     // or explicitly via `drop(et)`.
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct ExifTool {
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    stderr_receiver: Receiver<String>,
    child: Child,
}

impl ExifTool {
    /// Launches the `exiftool` process in stay-open mode using the default system path.
    ///
    /// This searches for `exiftool` in the directories specified by the system's `PATH`
    /// environment variable.
    ///
    /// # Errors
    ///
    /// Returns [`ExifToolError::ExifToolNotFound`] if the `exiftool` command cannot be found
    /// or if the process fails to start (e.g., due to permissions).
    /// Returns [`ExifToolError::Io`] if capturing the stdin/stdout/stderr pipes fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use exiftool::{ExifTool, ExifToolError};
    ///
    /// # fn main() -> Result<(), ExifToolError> {
    /// let et = ExifTool::new()?;
    /// println!("ExifTool process started successfully.");
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Result<Self, ExifToolError> {
        Self::with_executable(Path::new("exiftool"))
    }

    /// Launches `exiftool` from a specific executable path in stay-open mode.
    ///
    /// Use this if `exiftool` is not in the system's `PATH` or if you need to specify
    /// a particular version or location.
    ///
    /// # Arguments
    ///
    /// * `exiftool_path` - The path to the `exiftool` executable file.
    ///
    /// # Errors
    ///
    /// Returns [`ExifToolError::ExifToolNotFound`] if the specified `exiftool_path` does not exist,
    /// is not executable, or if the process fails to start.
    /// Returns [`ExifToolError::Io`] if capturing the stdin/stdout/stderr pipes fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use exiftool::{ExifTool, ExifToolError};
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), ExifToolError> {
    /// let path_to_exiftool = Path::new("/opt/local/bin/exiftool");
    /// let et = ExifTool::with_executable(path_to_exiftool)?;
    /// println!("ExifTool process started successfully from specific path.");
    /// # Ok(())
    /// # }
    /// ```
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
    /// This is the low-level method used by other helpers. It sends arguments line by line
    /// to the `exiftool` process's stdin, followed by `-execute\\n`, reads the response
    /// from stdout until `{ready}\\n`, and checks stderr for errors.
    ///
    /// **Note:** This method is typically not needed for common use cases. Prefer using
    /// methods like [`ExifTool::execute_lines`], [`ExifTool::json`], [`ExifTool::read_tag`],
    /// etc. unless you specifically need the raw byte output.
    ///
    /// # Arguments
    /// * `args` - A slice of string arguments to pass to `exiftool`. Do not include `-@ -`
    ///   or `-stay_open True`, as these are managed internally.
    ///
    /// # Errors
    /// Returns various [`ExifToolError`] variants, including:
    /// * [`ExifToolError::Io`]: If communication with the process fails.
    /// * [`ExifToolError::FileNotFound`]: If `exiftool` reports a file not found error.
    /// * [`ExifToolError::ExifToolProcess`]: If `exiftool` reports other errors on stderr.
    /// * [`ExifToolError::ProcessTerminated`]: If the process exits unexpectedly.
    /// * [`ExifToolError::StderrDisconnected`]: If the stderr monitoring fails.
    pub fn execute_raw(&mut self, args: &[&str]) -> Result<Vec<u8>, ExifToolError> {
        // 1. Clear any stale errors from previous commands
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

        // 5. Check for errors on stderr
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
                    warn!("ExifTool Warning - {}", err_line);
                }
            }
        }

        // If stderr contained only warnings or was empty, return the stdout bytes
        Ok(stdout_bytes)
    }

    /// Reads from stdout until the `exiftool` "{ready}" marker is found.
    /// Internal helper function.
    fn read_response_until_ready(&mut self) -> Result<Vec<u8>, ExifToolError> {
        let mut buffer = Vec::new();
        let ready_markers: &[&[u8]] = &[b"{ready}\n", b"{ready}\r\n"];

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

            // Check all possible markers
            for marker in ready_markers {
                if let Some(pos) = buffer.windows(marker.len()).position(|w| w == *marker) {
                    let data = buffer[..pos].to_vec();
                    buffer.drain(..pos + marker.len());
                    return Ok(data);
                }
            }
        }
    }

    /// Drains the stderr channel, collecting recent error messages.
    /// Internal helper function.
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
                        warn!("Stderr disconnected during polling after receiving some lines.");
                        break; // Return collected lines below
                    }
                }
            }
        }

        Ok(err_lines)
    }

    /// Sends the command to gracefully close the persistent exiftool process.
    ///
    /// This is called automatically when the [`ExifTool`] struct is dropped.
    /// There is usually no need to call this method directly.
    ///
    /// # Errors
    ///
    /// Returns [`ExifToolError::Io`] if writing the shutdown commands to the process fails.
    #[doc(hidden)] // Usually not called directly by users
    fn close(&mut self) -> Result<(), ExifToolError> {
        // Send the command to exit stay_open mode
        writeln!(self.stdin, "-stay_open")?;
        writeln!(self.stdin, "False")?;
        writeln!(self.stdin, "-execute")?;
        self.stdin.flush()?;
        Ok(())
    }

    // --- Public Helper Methods ---

    /// Executes an `exiftool` command and returns the standard output as lines of strings.
    ///
    /// Runs `exiftool {args...}` via the persistent process. Output is captured from stdout,
    /// split into lines, and returned as a `Vec<String>`.
    /// Standard error output from `exiftool` is checked for errors, and warnings are logged
    /// using the `log` crate.
    ///
    /// # Arguments
    ///
    /// * `args` - A slice of command-line arguments to pass to `exiftool`.
    ///   For example: `["-S", "-DateTimeOriginal", "image.jpg"]`.
    ///
    /// # Errors
    ///
    /// Returns an [`ExifToolError`] variant if the command fails, including:
    /// * [`ExifToolError::Io`]: If communication with the process fails.
    /// * [`ExifToolError::FileNotFound`]: If `exiftool` reports the file was not found.
    /// * [`ExifToolError::ExifToolProcess`]: If `exiftool` reports other errors on stderr.
    /// * [`ExifToolError::Utf8`]: If the output from `exiftool` is not valid UTF-8.
    /// * [`ExifToolError::ProcessTerminated`]: If the process exits unexpectedly.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use exiftool::{ExifTool, ExifToolError};
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), ExifToolError> {
    /// let mut exiftool = ExifTool::new()?;
    /// let path = Path::new("data/image.jpg");
    /// // Get Date/Time Original tag in standard (-S) format
    /// let output_lines = exiftool.execute_lines(&["-S", "-DateTimeOriginal", path.to_str().unwrap()])?;
    /// for line in output_lines {
    ///     println!("{}", line); // Example output: "DateTimeOriginal: 2023:10:27 10:00:00"
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn execute_lines(&mut self, args: &[&str]) -> Result<Vec<String>, ExifToolError> {
        let raw_output = self.execute_raw(args)?;
        let output_string = String::from_utf8(raw_output)?;
        Ok(output_string.lines().map(String::from).collect())
    }

    /// Executes a command with the `-json` argument and parses the output into a [`Value`].
    ///
    /// Runs `exiftool -json {args...}` via the persistent process.
    /// `exiftool` typically outputs a JSON array, even when processing a single file.
    /// This method parses the entire stdout content as a single JSON value.
    ///
    /// # Arguments
    ///
    /// * `args` - A slice of command-line arguments to pass to `exiftool`, *excluding* `-json`.
    ///   For example: `["-g1", "-Author", "image.jpg", "another.png"]`.
    ///
    /// # Errors
    ///
    /// Returns an [`ExifToolError`] variant if the command or parsing fails:
    /// * [`ExifToolError::Io`]: Communication failure.
    /// * [`ExifToolError::FileNotFound`]: File not found error from `exiftool`.
    /// * [`ExifToolError::ExifToolProcess`]: Other `exiftool` process errors.
    /// * [`ExifToolError::Json`]: The output was not valid JSON.
    /// * [`ExifToolError::UnexpectedFormat`]: If `exiftool` produces empty output when JSON was expected.
    /// * [`ExifToolError::ProcessTerminated`]: Unexpected process termination.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use exiftool::{ExifTool, ExifToolError};
    /// use std::path::Path;
    /// use serde_json::Value;
    ///
    /// # fn main() -> Result<(), ExifToolError> {
    /// let mut exiftool = ExifTool::new()?;
    /// let image_path = Path::new("data/image.jpg");
    /// let other_path = Path::new("data/another.png");
    ///
    /// // Get Author tag (grouped by -g1) for two files as JSON
    /// let json_output: Value = exiftool.json_execute(&[
    ///     "-g1",
    ///     "-Author",
    ///     image_path.to_str().unwrap(),
    ///     other_path.to_str().unwrap()
    /// ])?;
    ///
    /// if let Some(array) = json_output.as_array() {
    ///     for item in array {
    ///         println!("Metadata: {}", item);
    ///         // Example item: {"SourceFile": "data/image.jpg", "EXIF": {"Author": "Photographer"}}
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

    /// Reads metadata for multiple files, returning results as raw [`Value`]s.
    ///
    /// Runs `exiftool -json {extra_args...} {file_paths...}`.
    /// This is efficient for processing batches of files. `exiftool` outputs a JSON array
    /// where each element corresponds to a file path provided.
    ///
    /// # Arguments
    ///
    /// * `file_paths`: An iterator yielding items that can be referenced as a [`Path`]
    ///   (e.g., `Vec<PathBuf>`, `&[PathBuf]`, `Vec<&Path>`).
    /// * `extra_args`: Additional arguments to pass to `exiftool` before the file paths,
    ///   such as `-g1` (group tags), `-common` (extract common tags), `-DateTimeFormat`, etc.
    ///
    /// # Errors
    ///
    /// Returns an [`ExifToolError`] if the command or parsing fails. See [`ExifTool::json_execute`]
    /// for potential errors. Additionally, returns [`ExifToolError::UnexpectedFormat`] if the
    /// top-level JSON value returned by `exiftool` is not an array.
    /// Also returns [`ExifToolError::UnexpectedFormat`] if no files are passed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use exiftool::{ExifTool, ExifToolError};
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), ExifToolError> {
    /// let mut exiftool = ExifTool::new()?;
    /// let paths = [Path::new("image1.jpg"), Path::new("image2.png")];
    ///
    /// // Get common tags, grouped by family 1 (-g1) for both files
    /// let results = exiftool.json_batch(paths, &["-g1", "-common"])?;
    ///
    /// assert_eq!(results.len(), 2);
    /// println!("Metadata for first file: {}", results[0]);
    /// println!("Metadata for second file: {}", results[1]);
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
            return Err(ExifToolError::UnexpectedFormat {
                path: "".to_string(),
                command_args: extra_args.join(","),
            });
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

    /// Reads metadata for a single file, returning a raw [`Value`].
    ///
    /// Runs `exiftool -json {extra_args...} {file_path}`.
    /// Since `exiftool -json` typically returns an array even for a single file,
    /// this method extracts the *first* element from that array.
    ///
    /// # Arguments
    ///
    /// * `file_path`: The [`Path`] to the file to process.
    /// * `extra_args`: Additional arguments like `-g1`, `-common`, etc.
    ///
    /// # Errors
    ///
    /// Returns an [`ExifToolError`] if the command or parsing fails. See [`ExifTool::json_execute`]
    /// for potential errors. Additionally, returns [`ExifToolError::UnexpectedFormat`] if
    /// `exiftool` returns an empty array (which might happen if the file wasn't processed
    /// successfully, even if no stderr error occurred) or if the result is not an array.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use exiftool::{ExifTool, ExifToolError};
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), ExifToolError> {
    /// let mut exiftool = ExifTool::new()?;
    /// let path = Path::new("data/image.jpg");
    ///
    /// // Get common tags (-common) grouped by family 1 (-g1)
    /// let result = exiftool.json(path, &["-g1", "-common"])?;
    ///
    /// println!("Metadata: {}", result);
    /// // Example output: {"SourceFile": "data/image.jpg", "EXIF": {...}, "XMP": {...}, ...}
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

    /// Reads specific tags for a single file and deserializes the result into a struct `T`.
    ///
    /// Runs `exiftool -json {-TAG...} {file_path}`. The specified tags are requested,
    /// and the resulting JSON object (representing the file's metadata containing *only*
    /// those tags) is deserialized into the provided type `T`.
    ///
    /// The target type `T` must implement [`DeserializeOwned`]. Use `Option<V>` fields
    /// in your struct for tags that might be missing in the file.
    ///
    /// # Arguments
    ///
    /// * `file_path`: The [`Path`] to the file.
    /// * `tags`: A slice of tag names (e.g., `"Author"`, `"ImageWidth"`, `"GPSLatitude"`).
    ///   **Do not** include the leading `-` character.
    ///
    /// # Errors
    ///
    /// Returns an [`ExifToolError`] on failure:
    /// * Errors from [`ExifTool::json`]: Including file/process issues.
    /// * [`ExifToolError::Deserialization`]: If the JSON object returned by `exiftool`
    ///   (containing the requested tags) cannot be successfully deserialized into `T`.
    ///   The error provides context on *which field* failed using a JSON path.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use exiftool::{ExifTool, ExifToolError};
    /// use std::path::Path;
    /// use serde::Deserialize;
    ///
    /// // Define a struct matching the desired tags (case-insensitive with PascalCase default)
    /// #[derive(Deserialize, Debug)]
    /// #[serde(rename_all = "PascalCase")]
    /// struct LensInfo {
    ///     make: Option<String>, // Use Option for potentially missing tags
    ///     focal_length: Option<String>,
    ///     aperture: Option<f64>, // ExifTool often returns numbers as strings or numbers
    /// }
    ///
    /// # fn main() -> Result<(), ExifToolError> {
    /// let mut exiftool = ExifTool::new()?;
    /// let path = Path::new("photo.jpg");
    ///
    /// // Request specific tags
    /// let lens: LensInfo = exiftool.read_tags(path, &["Make", "FocalLength", "Aperture"])?;
    ///
    /// println!("Lens Info: {:?}", lens);
    /// if let Some(focal) = lens.focal_length {
    ///     println!("Focal Length: {}", focal);
    /// }
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

    /// Reads *all* available metadata for a single file and deserializes into struct `T`.
    ///
    /// Runs `exiftool -json {extra_args...} {file_path}`. The `extra_args` can be used
    /// to control the format and content of the JSON output (e.g., `-g1` for grouping,
    /// `-n` for numerical output). The entire resulting JSON object for the file is then
    /// deserialized into the target type `T`.
    ///
    /// The target type `T` must implement [`DeserializeOwned`]. Design your
    /// struct to match the JSON structure produced by `exiftool` with the chosen `extra_args`.
    /// Use `Option<V>` for fields that might not always be present.
    ///
    /// # Arguments
    ///
    /// * `file_path`: The [`Path`] to the file.
    /// * `extra_args`: A slice of arguments to pass to `exiftool` to control output format
    ///   (e.g., `&["-g1"]`, `&["-n", "-struct"]`).
    ///
    /// # Errors
    ///
    /// Returns an [`ExifToolError`] on failure:
    /// * Errors from [`ExifTool::json`]: Including file/process issues.
    /// * [`ExifToolError::Deserialization`]: If the JSON object returned by `exiftool`
    ///   cannot be successfully deserialized into `T`. The error provides context on
    ///   *which field* failed using a JSON path.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use exiftool::{ExifTool, ExifToolError};
    /// use std::path::Path;
    /// use serde::Deserialize;
    ///
    /// // Define a struct matching exiftool's -g1 (group 1) JSON output structure
    /// #[derive(Deserialize, Debug)]
    /// struct ExifData {
    ///     #[serde(rename = "SourceFile")]
    ///     source_file: String,
    ///     #[serde(rename = "EXIF")]
    ///     exif: Option<ExifGroup>,
    ///     #[serde(rename = "XMP")]
    ///     xmp: Option<XmpGroup>,
    ///     // Add other groups as needed (Composite, MakerNotes, etc.)
    /// }
    ///
    /// #[derive(Deserialize, Debug)]
    /// struct ExifGroup {
    ///     #[serde(rename = "Make")]
    ///     make: Option<String>,
    ///     #[serde(rename = "Model")]
    ///     model: Option<String>,
    ///     // ... other EXIF tags
    /// }
    ///
    /// #[derive(Deserialize, Debug)]
    /// struct XmpGroup {
    ///     #[serde(rename = "Creator")]
    ///     creator: Option<String>, // Example XMP tag
    ///     // ... other XMP tags
    /// }
    ///
    /// # fn main() -> Result<(), ExifToolError> {
    /// let mut exiftool = ExifTool::new()?;
    /// let path = Path::new("data/image.jpg");
    ///
    /// // Read metadata grouped by category (-g1)
    /// let metadata: ExifData = exiftool.read_metadata(path, &["-g1"])?;
    ///
    /// println!("Source File: {}", metadata.source_file);
    /// if let Some(exif) = metadata.exif {
    ///     println!("Make: {:?}", exif.make);
    /// }
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

    /// Reads a single tag's value as a raw [`Value`].
    ///
    /// Runs `exiftool -json -TAG {file_path}`. This efficiently requests only the specified tag.
    /// The method then extracts the value associated with that tag key from the resulting JSON object.
    ///
    /// # Arguments
    ///
    /// * `file_path`: The [`Path`] to the file.
    /// * `tag`: The name of the tag to read (e.g., `"Make"`, `"ImageWidth"`). Do not include the leading `-`.
    ///
    /// # Errors
    ///
    /// Returns an [`ExifToolError`] on failure:
    /// * Errors from [`ExifTool::json`]: Including file/process issues.
    /// * [`ExifToolError::TagNotFound`]: If the specified `tag` key is not present in the
    ///   JSON object returned by `exiftool`. This indicates the tag does not exist in the file's metadata.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use exiftool::{ExifTool, ExifToolError};
    /// use std::path::Path;
    /// use serde_json::Value;
    ///
    /// # fn main() -> Result<(), ExifToolError> {
    /// let mut et = ExifTool::new()?;
    /// let path = Path::new("data/image.jpg");
    ///
    /// let make_value: Value = et.json_tag(path, "Make")?;
    /// assert!(make_value.is_string());
    /// println!("Make JSON value: {}", make_value); // Output: "Huawei"
    ///
    /// let width_value: Value = et.json_tag(path, "ImageWidth")?;
    /// assert!(width_value.is_number());
    /// println!("Width JSON value: {}", width_value); // Output: 2688
    ///
    /// let missing_result = et.json_tag(path, "NonExistentTag");
    /// assert!(matches!(missing_result, Err(ExifToolError::TagNotFound { .. })));
    ///
    /// # Ok(())
    /// # }
    /// ```
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

    /// Reads a single tag and deserializes its value into a target type `T`.
    ///
    /// Runs `exiftool -json -TAG {file_path}`, extracts the tag's value, and attempts
    /// to deserialize that specific value (e.g., a JSON string, number, or boolean)
    /// into the requested Rust type `T`, which must implement [`DeserializeOwned`].
    ///
    /// This method intelligently handles missing tags when `T` is an `Option`:
    ///
    /// # Behavior
    ///
    /// *   **Tag Found, Deserializes Correctly:** Returns `Ok(T)` containing the value.
    /// *   **Tag Found, Deserialization Fails:** Returns `Err(ExifToolError::TagDeserialization)`
    ///     indicating a type mismatch between the tag's JSON value and `T`.
    /// *   **Tag Not Found, `T` is `Option<Inner>`:** Returns `Ok(T)` containing the `None` variant.
    ///     This allows gracefully handling potentially missing tags.
    /// *   **Tag Not Found, `T` is NOT `Option<Inner>`:** Returns `Err(ExifToolError::TagNotFound)`.
    ///     The tag was required but missing.
    /// *   **Other Errors:** Propagates errors from [`ExifTool::json_tag`] (e.g., file not found, process errors).
    ///
    /// # Arguments
    ///
    /// * `file_path`: The [`Path`] to the file.
    /// * `tag`: The name of the tag to read (e.g., `"Make"`, `"ImageWidth"`). Do not include the leading `-`.
    ///
    /// # Errors
    ///
    /// Returns [`ExifToolError`] as described above, including:
    /// * [`ExifToolError::TagNotFound`] - Can only happen when serializing into a non-Option field.
    /// * [`ExifToolError::TagDeserialization`]
    /// * Errors from the underlying [`ExifTool::json_tag`] call.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use exiftool::{ExifTool, ExifToolError};
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), ExifToolError> {
    /// let mut exiftool = ExifTool::new()?;
    /// let path = Path::new("data/image.jpg");
    ///
    /// // Read required tag (String) - Ok(String)
    /// let make: String = exiftool.read_tag(path, "Make")?;
    /// assert_eq!(make, "Huawei");
    ///
    /// // Read required tag (u32) - Ok(u32)
    /// let width: u32 = exiftool.read_tag(path, "ImageWidth")?;
    /// assert_eq!(width, 2688);
    ///
    /// // Read optional tag (Option<String>) that exists - Ok(Some(String))
    /// let model: Option<String> = exiftool.read_tag(path, "Model")?;
    /// assert!(model.is_some());
    ///
    /// // Read optional tag (Option<String>) that is missing - Ok(None)
    /// let comment: Option<String> = exiftool.read_tag(path, "UserComment")?;
    /// assert!(comment.is_none());
    ///
    /// // Read missing tag into required type (String) - Err(TagNotFound)
    /// let missing_req_result: Result<String, _> = exiftool.read_tag(path, "NonExistentTag");
    /// assert!(matches!(missing_req_result, Err(ExifToolError::TagNotFound { .. })));
    ///
    /// // Read existing tag (u32) into wrong type (String) - Err(TagDeserialization)
    /// let type_mismatch_result: Result<String, _> = exiftool.read_tag(path, "ImageWidth");
    /// assert!(matches!(type_mismatch_result, Err(ExifToolError::TagDeserialization { .. })));
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

    /// Reads a binary tag (like `ThumbnailImage`, `PreviewImage`) as raw bytes (`Vec<u8>`).
    ///
    /// Runs `exiftool -b -TAG {file_path}`. The `-b` option tells `exiftool` to output
    /// the binary data directly to standard output.
    ///
    /// # Arguments
    ///
    /// * `file_path`: The [`Path`] to the file.
    /// * `tag`: The name of the binary tag to read (e.g., `"ThumbnailImage"`, `"PreviewImage"`).
    ///   Do not include the leading `-`.
    ///
    /// # Errors
    ///
    /// Returns an [`ExifToolError`] on failure:
    /// * Errors from the underlying [`ExifTool::execute_raw`] call (IO, Process errors).
    /// * [`ExifToolError::TagNotFound`]: If `exiftool` returns *empty* output, which typically
    ///   indicates the binary tag was not found or was empty.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use exiftool::{ExifTool, ExifToolError};
    /// use std::path::Path;
    /// use std::fs;
    ///
    /// # fn main() -> Result<(), ExifToolError> {
    /// let mut et = ExifTool::new()?;
    /// let path = Path::new("data/image.jpg");
    ///
    /// let thumb_bytes = et.read_tag_binary(path, "ThumbnailImage")?;
    ///
    /// if !thumb_bytes.is_empty() {
    ///     println!("Read {} bytes for ThumbnailImage.", thumb_bytes.len());
    ///     // Optionally save or process the bytes
    ///     // fs::write("thumbnail.jpg", &thumb_bytes).map_err(ExifToolError::Io)?;
    /// } else {
    ///     println!("ThumbnailImage tag exists but is empty.");
    /// }
    ///
    /// // Try reading a non-existent binary tag
    /// let missing_result = et.read_tag_binary(path, "NonExistentBinaryTag");
    /// assert!(matches!(missing_result, Err(ExifToolError::TagNotFound { .. })));
    ///
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

    /// Writes a value (converted to a string) to a specific tag in a file's metadata.
    ///
    /// Runs `exiftool {-TAG=VALUE} {extra_args...} {file_path}`.
    /// The `value` provided will be converted to its string representation using [`ToString`]
    /// before being passed to `exiftool`.
    ///
    /// **Warning:** By default, `exiftool` creates a backup file by renaming the original
    /// file to `{filename}_original`. To prevent this and modify the file in place,
    /// include `"-overwrite_original"` in the `extra_args`. Use with caution.
    ///
    /// # Arguments
    ///
    /// * `file_path`: The [`Path`] to the file to modify.
    /// * `tag`: The name of the tag to write (e.g., `"Author"`, `"UserComment"`).
    ///   Do not include the leading `-`.
    /// * `value`: The value to write. Any type implementing [`ToString`] can be passed
    ///   (e.g., `&str`, `String`, `i32`, `f64`).
    /// * `extra_args`: A slice of additional arguments for `exiftool`, such as
    ///   `"-overwrite_original"` or `"-P"` (preserve modification date).
    ///
    /// # Errors
    ///
    /// Returns an [`ExifToolError`] on failure:
    /// * Errors from the underlying [`ExifTool::execute_raw`] call (e.g., [`ExifToolError::Io`],
    ///   [`ExifToolError::FileNotFound`], [`ExifToolError::ExifToolProcess`]).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use exiftool::{ExifTool, ExifToolError};
    /// use std::path::{Path, PathBuf};
    /// use std::fs;
    ///
    /// # fn setup_temp_image(name: &str) -> Result<PathBuf, ExifToolError> {
    /// #     let target = PathBuf::from("data").join(name);
    /// #     fs::copy("data/image.jpg", &target).map_err(ExifToolError::Io)?;
    /// #     Ok(target)
    /// # }
    /// # fn cleanup_temp_image(path: &Path) -> Result<(), ExifToolError> {
    /// #     fs::remove_file(path).map_err(ExifToolError::Io)?;
    /// #     let backup = path.with_extension("jpg_original");
    /// #     if backup.exists() { fs::remove_file(backup).map_err(ExifToolError::Io)?; }
    /// #     Ok(())
    /// # }
    ///
    /// # fn main() -> Result<(), ExifToolError> {
    /// let temp_path = setup_temp_image("write_test.jpg")?;
    /// let mut et = ExifTool::new()?;
    ///
    /// // Write a simple string tag
    /// let comment = "This comment was written by the Rust exiftool crate.";
    /// et.write_tag(&temp_path, "UserComment", comment, &[])?; // Creates backup
    ///
    /// // Read back to verify
    /// let read_comment: String = et.read_tag(&temp_path, "UserComment")?;
    /// assert_eq!(comment, read_comment);
    /// println!("Successfully wrote and verified UserComment.");
    ///
    /// // Write a tag and overwrite the original file
    /// let author = "Rust Programmer";
    /// et.write_tag(&temp_path, "Artist", author, &["-overwrite_original"])?;
    /// let read_author: String = et.read_tag(&temp_path, "Artist")?;
    /// assert_eq!(author, read_author);
    /// assert!(!temp_path.with_extension("jpg_original").exists(), "Backup should not exist");
    /// println!("Successfully wrote Artist tag with overwrite.");
    ///
    /// cleanup_temp_image(&temp_path)?;
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
        // Format the core argument: -TAG=VALUE
        let tag_arg = format!("-{}={}", tag, value_str);

        let path_str = file_path.to_string_lossy();

        // Assemble arguments: tag assignment first, then extra args, then file path
        let mut args = vec![tag_arg.as_str()];
        args.extend_from_slice(extra_args);
        args.push(path_str.as_ref());

        // Execute the command. The output (usually like "1 image files updated") is ignored.
        // Errors are checked via stderr within execute_raw.
        self.execute_raw(&args)?;
        Ok(())
    }

    /// Writes raw binary data to a specific tag (e.g., `ThumbnailImage`).
    ///
    /// This method is suitable for writing data like image thumbnails, previews, or other
    /// binary metadata fields. It works by writing the provided `data` to a temporary file
    /// and then telling `exiftool` to read the tag's value from that file using the
    /// `-TAG<=TEMPFILE` syntax.
    ///
    /// **Warning:** By default, `exiftool` creates a backup file (`{filename}_original`).
    /// To prevent this, include `"-overwrite_original"` in `extra_args`.
    ///
    /// # Arguments
    ///
    /// * `file_path`: The [`Path`] to the file to modify.
    /// * `tag`: The name of the binary tag to write (e.g., `"ThumbnailImage"`).
    ///   Do not include the leading `-`.
    /// * `data`: The binary data to write, provided as anything implementing `AsRef<[u8]>`
    ///   (e.g., `&[u8]`, `Vec<u8>`).
    /// * `extra_args`: A slice of additional arguments for `exiftool`, such as
    ///   `"-overwrite_original"` or `"-P"`.
    ///
    /// # Errors
    ///
    /// Returns an [`ExifToolError`] on failure:
    /// * [`ExifToolError::Io`]: If creating or writing to the temporary file fails, or if
    ///   communication with the process fails.
    /// * Errors from the underlying [`ExifTool::execute_raw`] call (e.g., [`ExifToolError::FileNotFound`],
    ///   [`ExifToolError::ExifToolProcess`]).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use exiftool::{ExifTool, ExifToolError};
    /// use std::path::{Path, PathBuf};
    /// use std::fs;
    ///
    /// # fn setup_temp_image(name: &str) -> Result<PathBuf, ExifToolError> {
    /// #     let target = PathBuf::from("data").join(name);
    /// #     fs::copy("data/image.jpg", &target).map_err(ExifToolError::Io)?;
    /// #     Ok(target)
    /// # }
    /// # fn cleanup_temp_image(path: &Path) -> Result<(), ExifToolError> {
    /// #     fs::remove_file(path).map_err(ExifToolError::Io)?;
    /// #     let backup = path.with_extension("jpg_original");
    /// #     if backup.exists() { fs::remove_file(backup).map_err(ExifToolError::Io)?; }
    /// #     Ok(())
    /// # }
    ///
    /// # fn main() -> Result<(), ExifToolError> {
    /// let temp_path = setup_temp_image("write_binary_test.jpg")?;
    /// let mut et = ExifTool::new()?;
    ///
    /// // Create some dummy binary data (e.g., a tiny placeholder thumbnail)
    /// let new_thumbnail_bytes: Vec<u8> = vec![0xFF, 0xD8, 0xFF, 0xD9]; // Minimal valid JPEG
    ///
    /// // Write the binary data to the ThumbnailImage tag, overwriting original
    /// et.write_tag_binary(&temp_path, "ThumbnailImage", &new_thumbnail_bytes, &["-overwrite_original"])?;
    ///
    /// // Read back to verify
    /// let read_thumb = et.read_tag_binary(&temp_path, "ThumbnailImage")?;
    /// assert_eq!(new_thumbnail_bytes, read_thumb);
    /// println!("Successfully wrote and verified binary ThumbnailImage tag.");
    ///
    /// cleanup_temp_image(&temp_path)?;
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
    /// if it hasn't terminated after a short grace period (implicit in `kill`).
    fn drop(&mut self) {
        // 1. Attempt graceful shutdown by sending exit commands.
        if let Err(e) = self.close() {
            // Log if closing failed, but proceed to kill anyway.
            warn!("Failed to send close command to exiftool process: {}", e);
        }

        // 2. Kill the process. This ensures cleanup even if graceful shutdown fails
        //    or hangs. `kill()` on Unix sends SIGKILL; on Windows, TerminateProcess.
        if let Err(e) = self.child.kill() {
            // Log if killing failed (e.g., process already dead).
            warn!(
                "Failed to kill exiftool process (may already be dead): {}",
                e
            );
        }

        // 3. Wait for the process to fully terminate and release resources.
        //    This prevents zombie processes. Ignore the result, as we've already
        //    attempted to kill it.
        match self.child.wait() {
            Ok(status) => {
                log::debug!("Exiftool process exited with status: {}", status);
            }
            Err(e) => {
                warn!("Failed to wait on exiftool child process: {}", e);
            }
        }
        log::debug!("ExifTool instance dropped and process cleanup attempted.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::g2::ExifData;
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
