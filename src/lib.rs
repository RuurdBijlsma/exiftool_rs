pub mod core;
mod error;
mod utils;
pub use core::exiftool::ExifTool;
pub use error::ExifToolError;

#[cfg(feature = "full-deserialize")]
pub mod parse_fn;
#[cfg(feature = "full-deserialize")]
pub mod structs;
#[cfg(feature = "full-deserialize")]
pub use structs::g2::ExifData;
