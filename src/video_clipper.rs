use crate::error::{VideoClipError, Result};
use crate::ffmpeg::FFmpegCommand;
use crate::time_parser::TimeParser;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipRequest {
    pub input_file: String,
    pub start_time: String,
    pub end_time: String,
    pub output_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipResult {
    pub input_file: String,
    pub output_file: String,
    pub start_seconds: f64,
    pub end_seconds: f64,
    pub duration: f64,
    pub file_size_mb: Option<f64>,
    pub command: String,
}

#[derive(Debug, Clone)]
pub struct VideoClipper {
    output_dir: PathBuf,
}

impl VideoClipper {
    pub fn new() -> Self {
        Self {
            output_dir: PathBuf::from("downloads"),
        }
    }
    
    pub fn with_output_dir(output_dir: impl AsRef<Path>) -> Self {
        Self {
            output_dir: output_dir.as_ref().to_path_buf(),
        }
    }
    
    pub fn ensure_output_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.output_dir)
            .map_err(|e| VideoClipError::IoError(e))
    }
    
    pub fn validate_input_file(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            // Try common locations
            let file_name = path.file_name()
                .ok_or_else(|| VideoClipError::InvalidPath(path.display().to_string()))?;
            
            let possible_paths = vec![
                PathBuf::from(format!("/workspaces/agentics-foundation-video-clip-rs/{}", file_name.to_string_lossy())),
                self.output_dir.join(file_name),
                path.to_path_buf(),
            ];
            
            for possible_path in &possible_paths {
                if possible_path.exists() {
                    return Ok(());
                }
            }
            
            return Err(VideoClipError::FileNotFound(path.display().to_string()));
        }
        Ok(())
    }
    
    pub fn generate_output_filename(&self, input_file: &Path, start_sec: f64, end_sec: f64) -> PathBuf {
        let stem = input_file.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("clip");
            
        let start_formatted = TimeParser::format_time(start_sec);
        let end_formatted = TimeParser::format_time(end_sec);
        
        let filename = format!("{}_clip_{}_to_{}.mp4", stem, start_formatted, end_formatted);
        self.output_dir.join(filename)
    }
    
    #[cfg(not(feature = "wasm"))]
    pub fn clip_video(&self, request: &ClipRequest) -> Result<ClipResult> {
        // Parse times
        let start_sec = TimeParser::parse_to_seconds(&request.start_time)?;
        let end_sec = TimeParser::parse_to_seconds(&request.end_time)?;
        let duration = TimeParser::validate_time_range(start_sec, end_sec)?;
        
        // Validate input file
        let input_path = Path::new(&request.input_file);
        self.validate_input_file(input_path)?;
        
        // Ensure output directory exists
        self.ensure_output_dir()?;
        
        // Generate output filename
        let output_path = if let Some(ref output_dir) = request.output_dir {
            let dir = Path::new(output_dir);
            fs::create_dir_all(dir)?;
            dir.join(self.generate_output_filename(input_path, start_sec, end_sec).file_name().unwrap())
        } else {
            self.generate_output_filename(input_path, start_sec, end_sec)
        };
        
        // Create and execute FFmpeg command
        let ffmpeg = FFmpegCommand::new(input_path, &output_path, start_sec, duration);
        let command_string = ffmpeg.get_command_string();
        
        ffmpeg.execute()?;
        
        // Get file size
        let file_size_mb = output_path.metadata()
            .ok()
            .map(|m| m.len() as f64 / (1024.0 * 1024.0));
        
        Ok(ClipResult {
            input_file: request.input_file.clone(),
            output_file: output_path.display().to_string(),
            start_seconds: start_sec,
            end_seconds: end_sec,
            duration,
            file_size_mb,
            command: command_string,
        })
    }
    
    #[cfg(feature = "wasm")]
    pub fn prepare_clip_command(&self, request: &ClipRequest) -> Result<ClipResult> {
        // Parse times
        let start_sec = TimeParser::parse_to_seconds(&request.start_time)?;
        let end_sec = TimeParser::parse_to_seconds(&request.end_time)?;
        let duration = TimeParser::validate_time_range(start_sec, end_sec)?;
        
        let input_path = Path::new(&request.input_file);
        let output_path = self.generate_output_filename(input_path, start_sec, end_sec);
        
        let ffmpeg = FFmpegCommand::new(input_path, &output_path, start_sec, duration);
        let command_string = ffmpeg.get_command_string();
        
        Ok(ClipResult {
            input_file: request.input_file.clone(),
            output_file: output_path.display().to_string(),
            start_seconds: start_sec,
            end_seconds: end_sec,
            duration,
            file_size_mb: None,
            command: command_string,
        })
    }
}

impl Default for VideoClipper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_output_filename_generation() {
        let clipper = VideoClipper::new();
        let output = clipper.generate_output_filename(
            Path::new("test_video.mp4"),
            125.0,  // 2:05
            190.0   // 3:10
        );
        
        assert!(output.to_string_lossy().contains("test_video_clip_02-05_to_03-10.mp4"));
    }
    
    #[test]
    fn test_time_range_validation() {
        let clipper = VideoClipper::new();
        let request = ClipRequest {
            input_file: "test.mp4".to_string(),
            start_time: "1:00".to_string(),
            end_time: "2:00".to_string(),
            output_dir: None,
        };
        
        #[cfg(feature = "wasm")]
        {
            let result = clipper.prepare_clip_command(&request);
            assert!(result.is_ok());
            if let Ok(clip_result) = result {
                assert_eq!(clip_result.duration, 60.0);
            }
        }
    }
}