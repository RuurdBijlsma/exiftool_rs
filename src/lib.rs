//! # ExifTool
//!
//! A Rust wrapper library for Phil Harvey's ExifTool command-line application.
//!
//! This library allows you to interact with ExifTool, enabling reading and writing
//! metadata tags for a wide variety of file types (images, videos, audio, documents).
//!
//! It maintains a long-running ExifTool process in stay-open mode for efficiency when
//! processing multiple files or commands.
//!
//! ## Basic Usage
//!
//! ```no_run
//! use exiftool::{ExifTool, ExifToolError};
//! use std::path::Path;
//!
//! fn main() -> Result<(), ExifToolError> {
//!     let mut exiftool = ExifTool::new()?; // Starts the background ExifTool process
//!     let image_path = Path::new("path/to/your/image.jpg");
//!
//!     // Read a specific tag as a JSON Value
//!     let author_value = exiftool.json_tag(image_path, "Author")?;
//!     if let Some(author) = author_value.as_str() {
//!         println!("Author: {}", author);
//!     }
//!
//!     // Read all metadata as a JSON Value (grouped by category)
//!     let metadata_json = exiftool.json(image_path, &["-g1"])?;
//!     println!("Metadata JSON: {}", metadata_json);
//!
//!     // Write a tag
//!     exiftool.write_tag(image_path, "UserComment", "This is a test comment", &["-overwrite_original"])?;
//!
//!     // Read binary data (e.g., thumbnail)
//!     let thumbnail_bytes = exiftool.read_tag_binary(image_path, "ThumbnailImage")?;
//!     println!("Read {} bytes for thumbnail", thumbnail_bytes.len());
//!
//!     // Remember ExifTool process closes when `exiftool` variable goes out of scope (Drop).
//!     Ok(())
//! }
//! ```
//!
//!
//! ```no_run
//! use exiftool::{ExifTool, ExifToolError};
//! use std::path::Path;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize, Debug)]
//! struct ImageMetadata {
//!     #[serde(rename = "FileName")]
//!     file_name: String,
//!     #[serde(rename = "ImageWidth")]
//!     width: u32,
//!     #[serde(rename = "ImageHeight")]
//!     height: u32,
//! }
//!
//! fn main() -> Result<(), ExifToolError> {
//!     let mut exiftool = ExifTool::new()?;
//!     let image_path = Path::new("path/to/your/image.jpg");
//!
//!     // Read specific tags and deserialize into a struct
//!     let partial_meta: ImageMetadata = exiftool.read_tags(
//!         image_path,
//!         &["FileName", "ImageWidth", "ImageHeight"]
//!     )?;
//!     println!("Partial Metadata: {:?}", partial_meta);
//!
//!     // Read all metadata (grouped) and deserialize
//!     // Note: Requires a struct matching ExifTool's -g/-G output structure
//!     // let full_meta: YourFullMetadataStruct = exiftool.read_metadata(image_path, &["-g1"])?;
//!     // println!("Full Metadata: {:?}", full_meta);
//!
//!     // Read a single tag and deserialize
//!     let author: String = exiftool.read_tag(image_path, "Author")?;
//!     println!("Author: {}", author);
//!
//!     Ok(())
//! }
//! ```

// Public API
mod error;
mod exiftool;

pub use error::ExifToolError;
pub use exiftool::ExifTool;

pub mod parse_fn;
pub mod structs;
pub use structs::g2::ExifData;

mod utils;
