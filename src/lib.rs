mod error;
mod exiftool;
mod parse;
pub mod parse_fn;
pub mod structs;
mod utils;

pub use error::ExifToolError;
pub use exiftool::ExifTool;
pub use parse::parse_output::parse_output;
