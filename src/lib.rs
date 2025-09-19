pub mod error;
pub mod time_parser;
pub mod video_clipper;
pub mod ffmpeg;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use error::{VideoClipError, Result};
pub use time_parser::TimeParser;
pub use video_clipper::{VideoClipper, ClipRequest, ClipResult};
pub use ffmpeg::{FFmpegCommand, AudioCodec};

#[cfg(feature = "wasm")]
pub use wasm::*;