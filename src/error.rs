use thiserror::Error;

#[derive(Error, Debug)]
pub enum VideoClipError {
    #[error("Invalid time format: {0}")]
    InvalidTimeFormat(String),
    
    #[error("End time ({end}) must be after start time ({start})")]
    InvalidTimeRange { start: f64, end: f64 },
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("FFmpeg not installed or not in PATH")]
    FFmpegNotFound,
    
    #[error("FFmpeg execution failed: {0}")]
    FFmpegError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Invalid file path: {0}")]
    InvalidPath(String),
    
    #[error("WASM error: {0}")]
    #[cfg(feature = "wasm")]
    WasmError(String),
}

pub type Result<T> = std::result::Result<T, VideoClipError>;