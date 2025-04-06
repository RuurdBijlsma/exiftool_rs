# ExifTool Rust Wrapper

[![Crates.io](https://img.shields.io/crates/v/exiftool.svg)](https://crates.io/crates/exiftool) <!-- TODO: Update badge when published -->
[![Docs.rs](https://docs.rs/exiftool/badge.svg)](https://docs.rs/exiftool) <!-- TODO: Update badge when published -->
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Rust wrapper for Phil Harvey's [ExifTool](https://exiftool.org/) command-line application.

This crate interacts with a persistent `exiftool` process using the `-stay_open` argument, significantly reducing
overhead compared to spawning a new process for each command.

**Note:** This crate assumes that the `exiftool` command-line executable is already installed on your system and
available in your system's PATH.

## Features

* **Fast:** Uses a long-running `exiftool` process for minimal overhead per command.
* **Robust:** Provides a typed Rust interface and error handling (`ExifToolError`).
* **Flexible JSON Output:** Easily retrieve metadata as `serde_json::Value` using the `-json` argument.
* **Typed Struct Deserialization:** Includes helper functions and pre-defined structs (`ExifData`) to parse JSON output
  into convenient Rust types.
* **Binary Data Extraction:** Directly extract binary data like thumbnails or previews.
* **Raw Command Execution:** Allows executing arbitrary `exiftool` commands.
* **Cross-Platform:** CI Tests on Windows, Linux, and Mac.

## Prerequisites

You must have Phil Harvey's ExifTool command-line utility installed and accessible in your system's PATH.

* **Official Website & Installation:** [https://exiftool.org/](https://exiftool.org/)
* **macOS (Homebrew):** `brew install exiftool`
* **Debian/Ubuntu:** `sudo apt install libimage-exiftool-perl`
* **Windows:** Download the Windows Executable from the official website and ensure its location is in your PATH
  environment variable.

Verify your installation by typing `exiftool` in your terminal.

## Usage

Usage Examples

### Initialize ExifTool

The ExifTool instance keeps the background process alive. It will be automatically shut down when the instance goes out
of scope (using Drop).

The `file_metadata` method is convenient for getting the JSON output for one file.

```rust
use exiftool::{ExifTool, ExifToolError};
use std::path::Path;

fn main() -> Result<(), ExifToolError> {
    let mut exiftool = ExifTool::new()?;
    let file = Path::new("file.jpg");
    // Optionally pass "-g2" to have the JSON output grouped by category.
    let value = exiftool.file_metadata(file, &["-g2"])?;
    dbg!(&value);
    Ok(())
}
```

### Get Metadata as JSON for Multiple Files

Use `execute_json` for processing multiple files in one command. The result is a JSON array.

```rust
use exiftool::{ExifTool, ExifToolError};
use serde_json::Value;

fn main() -> Result<(), ExifToolError> {
    let mut exiftool = ExifTool::new()?;
    let files = ["test_data/IMG_20170801_162043.jpg", "test_data/another_image.jpg"]; // TODO: Replace

    // Pass file paths as arguments
    let metadata_list: Value = exiftool.execute_json(&files)?;

    if let Some(array) = metadata_list.as_array() {
        for item in array {
            println!("Metadata for {}:\n{:#?}", item.get("SourceFile").and_then(|v| v.as_str()).unwrap_or("unknown"), item);
        }
    }

    Ok(())
}
```

### Parse JSON Output into Structs

Use the parse_output helper and the provided ExifData struct (designed for use with -g2) for typed access.

```rust
use exiftool::{ExifTool, ExifToolError};
use exiftool::structs::g2::ExifData;
use exiftool::parse::parse_output;
use std::path::Path;

fn main() -> Result<(), ExifToolError> {
    let mut exiftool = ExifTool::new()?;
    let file = Path::new("image.jpg");

    // Use -g2 for the structure expected by the ExifData type
    let json_value = exiftool.file_metadata(file, &["-g2"])?;

    // Parse the JSON Value into our struct
    let exif_data: ExifData = parse_output(&json_value)?;

    println!("Parsed Metadata:\n{:#?}", exif_data);

    if let Some(image_meta) = exif_data.image {
        println!("Aperture: {:?}", image_meta.aperture);
    }

    Ok(())
}
```

### Extract Binary Data

Use `binary_field` to get the raw bytes of a tag.

```rust
use exiftool::{ExifTool, ExifToolError};
use std::path::Path;
use std::fs;

fn main() -> Result<(), ExifToolError> {
    let mut exiftool = ExifTool::new()?;
    let file = Path::new("test_data/image.jpg");

    // Extract the thumbnail image
    let thumb_bytes = exiftool.binary_field(file, "ThumbnailImage")?;

    println!("Read {} bytes for ThumbnailImage", thumb_bytes.len());

    // Optional: Save the thumbnail
    fs::write("thumbnail.jpg", &thumb_bytes)?;

    Ok(())
}
```

### Execute Raw Commands

For commands not covered by helpers, use execute_raw.

```rust
use exiftool::{ExifTool, ExifToolError};
use std::path::Path;

fn main() -> Result<(), ExifToolError> {
    let mut exiftool = ExifTool::new()?;
    let file = "test_data/IMG_20170801_162043.jpg"; // TODO: Replace

    let args = &["-Author='Ruurd'", file];

    let output_bytes = exiftool.execute_raw(args)?;
    let output_string = String::from_utf8_lossy(&output_bytes);

    println!("Output: {}", output_string);
    // Output: 1 image files updated

    Ok(())
}
```

### Error Handling

All functions return Result<_, ExifToolError>. The ExifToolError enum covers various issues,
see [error.rs](src/error.rs).

## Provided Structs (ExifData)

This crate includes exiftool::structs::g2::ExifData which maps many common fields output by exiftool -g2 -json. This is
useful for quickly accessing typed data for standard image and video metadata. See
the [structs/g2.rs](src/structs/g2.rs) file for details
on the fields. Remember to pass "-g2" when calling file_metadata or execute_json if you intend to parse into ExifData.

## Performance

By keeping a single exiftool process running (-stay_open True -@ -), this wrapper avoids the significant startup cost
associated with launching exiftool for every command, making it suitable for batch processing or applications requiring
frequent metadata access.