# ExifTool Rust Wrapper

[![Crates.io](https://img.shields.io/crates/v/exiftool.svg)](https://crates.io/crates/exiftool)
[![Docs.rs](https://docs.rs/exiftool/badge.svg)](https://docs.rs/exiftool)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE.md)

Rust wrapper for Phil Harvey's [ExifTool](https://exiftool.org/) command-line application.

This crate interacts with a persistent `exiftool` process using the `-stay_open` argument, significantly reducing
overhead compared to spawning a new process for each command.

**Note:** This crate assumes that the `exiftool` command-line executable is available already, via PATH or by passing an
executable.

## Features

* **🚀 Fast:** Uses a long-running `exiftool` process (`-stay_open`) for minimal overhead per command.
* **🦀 Idiomatic Rust:** Provides a typed Rust interface, comprehensive error handling (`ExifToolError`), and leverages
  `serde` for flexible deserialization.
* **✅ Robust:** Includes extensive tests and CI across Windows, Linux, and macOS.
* **🛠️ Flexible:**
    * Read/Write string and binary tags.
    * Retrieve metadata as structured JSON (`serde_json::Value`).
    * Deserialize JSON output directly into your own Rust structs or use the provided `ExifData`.
    * Execute lower-level commands when needed.

## Prerequisites

You must have Phil Harvey's ExifTool command-line utility installed and accessible in your system's PATH.

* **Official Website & Installation:** [https://exiftool.org/](https://exiftool.org/)
* **macOS (Homebrew):** `brew install exiftool`
* **Debian/Ubuntu:** `sudo apt install libimage-exiftool-perl`
* **Windows:** Download the Windows Executable from the official website and ensure its location is in your PATH
  environment variable.

Verify your installation by typing `exiftool -ver` in your terminal.

## Usage Examples

### Read a Single Tag

```rust
use exiftool::{ExifTool, ExifToolError};
use std::path::Path;

fn main() -> Result<(), ExifToolError> {
    let mut exiftool = ExifTool::new()?;
    let path = Path::new("data/image.jpg");

    // Read a tag (String)
    let make: String = exiftool.read_tag(path, "Make")?;
    println!("Make (String): {}", make); // Output: Make (String): Huawei

    // Read a required tag (u32)
    let width: u32 = exiftool.read_tag(path, "ImageWidth")?;
    println!("Width (u32): {}", width); // Output: Width (u32): 2688

    // Read an optional tag that is missing
    let desc: Option<String> = exiftool.read_tag(path, "ImageDescription")?;
    println!("Description: {:?}", desc); // Output: Description: None

    Ok(())
}
```

### Read All Metadata (as JSON `Value`)

```rust
use exiftool::{ExifTool, ExifToolError};
use std::path::Path;

fn main() -> Result<(), ExifToolError> {
    let mut exiftool = ExifTool::new()?;
    let path = Path::new("data/image.jpg");

    // Get all metadata, grouped by category (image, audio, video, camera, etc.)
    let metadata_json = exiftool.json(path, &["-g2"])?;

    println!("All Metadata JSON (-g1 -common):\n{:#}", metadata_json);

    Ok(())
}
```

### Read and Deserialize All Metadata into a Struct.

There's a provided struct (`ExifData`) for dealing with common fields, if you want that type safety. `-g2` has to be
used to use this struct.

```rust
use exiftool::{ExifTool, ExifToolError, ExifData};
use std::path::Path;

fn main() -> Result<(), ExifToolError> {
    let mut exiftool = ExifTool::new()?;
    let path = Path::new("data/image.jpg");

    // Use -g2 for the structure expected by the ExifData type
    let exif_data: ExifData = exiftool.read_metadata(path, &["-g2"])?;

    println!("Parsed ExifData:\n{:#?}", exif_data);

    if let Some(camera_meta) = exif_data.camera {
        println!("Camera Make: {:?}", camera_meta.make);
        println!("Camera Model: {:?}", camera_meta.model);
    }
    if let Some(other_meta) = exif_data.other {
        println!("File Name: {:?}", other_meta.file_name);
        println!("MIME Type: {:?}", other_meta.mime_type);
    }

    Ok(())
}
```

### Read Metadata for Multiple Files (Batch)

```rust
use exiftool::{ExifTool, ExifToolError};
use std::path::Path;

fn main() -> Result<(), ExifToolError> {
    let mut exiftool = ExifTool::new()?;
    let paths = [
        Path::new("data/image.jpg"),
        Path::new("data/other_images/jpg/gps/DSCN0010.jpg")
    ];

    // Get specific tags for multiple files, if you want all tags, leave the `extra_args` empty.
    let results = exiftool.json_batch(&paths, &["-FileName", "-FileSize", "-ImageWidth"])?;

    for metadata_value in results {
        println!("--- File: {} ---", metadata_value.get("SourceFile").and_then(|v| v.as_str()).unwrap_or("N/A"));
        println!("{:#}", metadata_value);
    }

    Ok(())
}
```

### Read Binary Data (e.g., Thumbnail)

```rust
use exiftool::{ExifTool, ExifToolError};
use std::path::Path;
use std::fs;

fn main() -> Result<(), ExifToolError> {
    let mut exiftool = ExifTool::new()?;
    let path = Path::new("data/image.jpg");

    // Extract the thumbnail image
    let thumb_bytes = exiftool.read_tag_binary(path, "ThumbnailImage")?;
    println!("Read {} bytes for ThumbnailImage", thumb_bytes.len());
    // Optional: Save the thumbnail
    fs::write("thumbnail.jpg", &thumb_bytes)?;
    assert!(!thumb_bytes.is_empty());

    Ok(())
}
```

### Write a Tag

**Warning:** ExifTool creates a backup file named `{filename}_original` when writing.

```rust
use exiftool::{ExifTool, ExifToolError};
use std::path::{Path, PathBuf};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> { // Using Box<dyn Error> for example simplicity
    let mut exiftool = ExifTool::new()?;
    let source_path = Path::new("data/image.jpg");

    let new_comment = "Written by exiftool-rs test!";

    println!("Writing UserComment to: {}", source_path.display());
    exiftool.write_tag(&source_path, "UserComment", new_comment, &[])?;
    println!("Write successful (check file metadata externally).");

    let read_comment: String = exiftool.read_tag(&source_path, "UserComment")?;
    assert_eq!(read_comment, new_comment);
    println!("Verification successful!");

    Ok(())
}

```

### Write Binary Data

Uses a temporary file internally. Also creates `{filename}_original` as backup.

```rust
use exiftool::{ExifTool, ExifToolError};
use std::path::{Path, PathBuf};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut exiftool = ExifTool::new()?;
    let source_path = Path::new("data/image.jpg");

    // Create some dummy binary data (e.g., a tiny valid JPEG)
    let dummy_thumb = b"\xFF\xD8\xFF\xE0\x00\x10JFIF\x00\x01\x01\x00\x00\x01\x00\x01\x00\x00\xFF\xDB\x00C\x00\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\x01\xFF\xC0\x00\x11\x08\x00\x01\x00\x01\x03\x01\x22\x00\x02\x11\x01\x03\x11\x01\xFF\xC4\x00\x15\x00\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xFF\xDA\x00\x0C\x03\x01\x00\x02\x11\x03\x11\x00\x3F\x00\xA8\xFF\xD9";

    println!("Writing binary ThumbnailImage to: {}", source_path.display());
    exiftool.write_tag_binary(&source_path, "ThumbnailImage", &dummy_thumb[..], &[])?;
    println!("Binary write successful.");

    // Verify (Optional)
    let read_thumb = exiftool.read_tag_binary(&source_path, "ThumbnailImage")?;
    assert_eq!(read_thumb, dummy_thumb);
    println!("Binary verification successful!");

    Ok(())
}
```

### Execute Lower-Level Commands

For commands not covered by helpers, use `execute_lines` (string lines), `json_execute` (json value), or `execute_raw` (
bytes).

```rust
use exiftool::{ExifTool, ExifToolError};

fn main() -> Result<(), ExifToolError> {
    let mut exiftool = ExifTool::new()?;
    let path = "data/image.jpg";

    // Example: Get verbose, structured output (-S) as lines
    let args = &["-S", "-Make", "-Model", path];
    let output_lines = exiftool.execute_lines(args)?;

    println!("execute_lines Output:");
    for line in output_lines {
        println!("> {}", line);
    }
    // Output:
    // > Make: Huawei
    // > Model: Nexus 6P

    Ok(())
}
```

## Provided Struct (`ExifData`)

This crate provides `exiftool::ExifData`. This struct maps many common fields
output by `exiftool -g2 -json`. It's useful for accessing typed data for standard image and video metadata.

* See the [structs/g2.rs](https://docs.rs/exiftool/latest/exiftool/structs/g2/struct.ExifData.html) file for details on the available fields.
* Remember to pass `"-g2"` when calling `read_metadata`.

## Error Handling

All potentially failing operations return `Result<_, ExifToolError>`. The [`ExifToolError`](https://docs.rs/exiftool/latest/exiftool/enum.ExifToolError.html) enum covers
various issues, including:

* IO errors communicating with the process.
* ExifTool executable not found.
* Errors reported by the ExifTool process (e.g., file not found, invalid arguments).
* JSON parsing/deserialization errors.
* Tag not found errors.
* Process termination issues.

## Performance

By keeping a single exiftool process running (-stay_open True -@ -), this wrapper avoids the significant startup cost
associated with launching exiftool for every command, making it suitable for batch processing or applications requiring
frequent metadata access.