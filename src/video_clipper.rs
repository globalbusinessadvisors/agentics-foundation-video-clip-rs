use crate::error::{VideoClipError, Result};
use crate::ffmpeg::FFmpegCommand;
use crate::time_parser::TimeParser;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Video clipping request containing input parameters
/// Used to specify which video to clip and the time range

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
        
        #[cfg(not(feature = "wasm"))]
        ffmpeg.execute()?;
        
        // Get file size (only in non-WASM environments)
        #[cfg(not(feature = "wasm"))]
        let file_size_mb = output_path.metadata()
            .ok()
            .map(|m| m.len() as f64 / (1024.0 * 1024.0));
        
        #[cfg(feature = "wasm")]
        let file_size_mb = None;
        
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
    
    mod filename_generation_tests {
        use super::*;
        
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
        fn test_filename_with_special_characters() {
            let clipper = VideoClipper::new();
            let output = clipper.generate_output_filename(
                Path::new("my video (2023).mov"),
                0.0,
                30.0
            );
            
            let filename = output.file_name().unwrap().to_string_lossy();
            assert!(filename.contains("my video (2023)_clip"));
            assert!(filename.ends_with(".mp4"));
        }
        
        #[test]
        fn test_filename_without_extension() {
            let clipper = VideoClipper::new();
            let output = clipper.generate_output_filename(
                Path::new("video_file"),
                60.0,
                120.0
            );
            
            let filename = output.file_name().unwrap().to_string_lossy();
            assert!(filename.contains("video_file_clip_01-00_to_02-00.mp4"));
        }
        
        #[test]
        fn test_filename_with_custom_output_dir() {
            let temp_dir = tempdir().unwrap();
            let clipper = VideoClipper::with_output_dir(temp_dir.path());
            let output = clipper.generate_output_filename(
                Path::new("test.mp4"),
                0.0,
                30.0
            );
            
            assert_eq!(output.parent().unwrap(), temp_dir.path());
        }
    }
    
    mod validation_tests {
        use super::*;
        
        #[test]
        fn test_validate_nonexistent_file() {
            let clipper = VideoClipper::new();
            let result = clipper.validate_input_file(Path::new("nonexistent.mp4"));
            assert!(result.is_err());
            
            match result {
                Err(crate::VideoClipError::FileNotFound(_)) => {},
                _ => panic!("Expected FileNotFound error"),
            }
        }
        
        #[test]
        fn test_ensure_output_dir_creation() {
            let temp_dir = tempdir().unwrap();
            let output_path = temp_dir.path().join("nested").join("output");
            let clipper = VideoClipper::with_output_dir(&output_path);
            
            assert!(!output_path.exists());
            clipper.ensure_output_dir().unwrap();
            assert!(output_path.exists());
        }
    }
    
    mod clip_request_tests {
        use super::*;
        
        #[test]
        fn test_clip_request_serialization() {
            let request = ClipRequest {
                input_file: "test.mp4".to_string(),
                start_time: "1:00".to_string(),
                end_time: "2:00".to_string(),
                output_dir: Some("/tmp".to_string()),
            };
            
            let json = serde_json::to_string(&request).unwrap();
            let deserialized: ClipRequest = serde_json::from_str(&json).unwrap();
            
            assert_eq!(request.input_file, deserialized.input_file);
            assert_eq!(request.start_time, deserialized.start_time);
            assert_eq!(request.end_time, deserialized.end_time);
            assert_eq!(request.output_dir, deserialized.output_dir);
        }
        
        #[test]
        fn test_clip_result_serialization() {
            let result = ClipResult {
                input_file: "test.mp4".to_string(),
                output_file: "output.mp4".to_string(),
                start_seconds: 60.0,
                end_seconds: 120.0,
                duration: 60.0,
                file_size_mb: Some(15.5),
                command: "ffmpeg -i test.mp4 -ss 60 -t 60 -c copy output.mp4".to_string(),
            };
            
            let json = serde_json::to_string(&result).unwrap();
            let deserialized: ClipResult = serde_json::from_str(&json).unwrap();
            
            assert_eq!(result.duration, deserialized.duration);
            assert_eq!(result.file_size_mb, deserialized.file_size_mb);
        }
    }
    
    #[cfg(feature = "wasm")]
    mod wasm_tests {
        use super::*;
        
        #[test]
        fn test_prepare_clip_command() {
            let clipper = VideoClipper::new();
            let request = ClipRequest {
                input_file: "test.mp4".to_string(),
                start_time: "1:00".to_string(),
                end_time: "2:00".to_string(),
                output_dir: None,
            };
            
            let result = clipper.prepare_clip_command(&request);
            assert!(result.is_ok());
            
            if let Ok(clip_result) = result {
                assert_eq!(clip_result.duration, 60.0);
                assert_eq!(clip_result.start_seconds, 60.0);
                assert_eq!(clip_result.end_seconds, 120.0);
                assert!(clip_result.command.contains("ffmpeg"));
                assert!(clip_result.command.contains("-ss 60"));
                assert!(clip_result.command.contains("-t 60"));
            }
        }
        
        #[test]
        fn test_prepare_clip_invalid_times() {
            let clipper = VideoClipper::new();
            let request = ClipRequest {
                input_file: "test.mp4".to_string(),
                start_time: "2:00".to_string(),
                end_time: "1:00".to_string(),  // End before start
                output_dir: None,
            };
            
            let result = clipper.prepare_clip_command(&request);
            assert!(result.is_err());
        }
    }
    
    #[cfg(not(feature = "wasm"))]
    mod native_tests {
        use super::*;
        use std::fs::File;
        use std::io::Write;
        
        #[test]
        fn test_clip_video_with_missing_file() {
            let clipper = VideoClipper::new();
            let request = ClipRequest {
                input_file: "nonexistent.mp4".to_string(),
                start_time: "0:00".to_string(),
                end_time: "0:30".to_string(),
                output_dir: None,
            };
            
            let result = clipper.clip_video(&request);
            assert!(result.is_err());
        }
        
        #[test]
        fn test_output_directory_creation() {
            let temp_dir = tempdir().unwrap();
            let output_dir = temp_dir.path().join("custom_output");
            
            let request = ClipRequest {
                input_file: "test.mp4".to_string(),
                start_time: "0:00".to_string(),
                end_time: "0:30".to_string(),
                output_dir: Some(output_dir.to_string_lossy().to_string()),
            };
            
            // Create a dummy input file
            let input_file = temp_dir.path().join("test.mp4");
            File::create(&input_file).unwrap();
            
            let clipper = VideoClipper::new();
            // This will fail due to FFmpeg, but it should create the directory
            let _ = clipper.clip_video(&request);
            
            // The directory should still be created even if FFmpeg fails
            assert!(output_dir.exists());
        }
    }
    
    mod integration_tests {
        use super::*;
        
        #[test]
        fn test_full_workflow_preparation() {
            let clipper = VideoClipper::new();
            let request = ClipRequest {
                input_file: "sample.mp4".to_string(),
                start_time: "0:30".to_string(),
                end_time: "1:45".to_string(),
                output_dir: None,
            };
            
            // Test time parsing
            let start_sec = crate::TimeParser::parse_to_seconds(&request.start_time).unwrap();
            let end_sec = crate::TimeParser::parse_to_seconds(&request.end_time).unwrap();
            let duration = crate::TimeParser::validate_time_range(start_sec, end_sec).unwrap();
            
            assert_eq!(start_sec, 30.0);
            assert_eq!(end_sec, 105.0);
            assert_eq!(duration, 75.0);
            
            // Test filename generation
            let output_path = clipper.generate_output_filename(
                Path::new(&request.input_file),
                start_sec,
                end_sec
            );
            
            let filename = output_path.file_name().unwrap().to_string_lossy();
            assert!(filename.contains("sample_clip_00-30_to_01-45.mp4"));
        }
    }
}