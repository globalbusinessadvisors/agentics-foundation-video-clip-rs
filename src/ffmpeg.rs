use crate::error::{VideoClipError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(not(feature = "wasm"))]
use std::process::Output;

/// FFmpeg command builder and executor
/// Handles the construction and execution of FFmpeg commands for video clipping
/// Supports both native execution and WASM command string generation

#[derive(Debug, Clone)]
pub struct FFmpegCommand {
    input: PathBuf,
    output: PathBuf,
    start_time: f64,
    duration: f64,
}

impl FFmpegCommand {
    pub fn new(input: impl AsRef<Path>, output: impl AsRef<Path>, start_time: f64, duration: f64) -> Self {
        Self {
            input: input.as_ref().to_path_buf(),
            output: output.as_ref().to_path_buf(),
            start_time,
            duration,
        }
    }
    
    pub fn check_ffmpeg_installed() -> Result<()> {
        let output = Command::new("ffmpeg")
            .arg("-version")
            .output();
            
        match output {
            Ok(_) => Ok(()),
            Err(_) => Err(VideoClipError::FFmpegNotFound),
        }
    }
    
    pub fn build_command(&self) -> Command {
        let mut cmd = Command::new("ffmpeg");
        
        cmd.arg("-i")
            .arg(&self.input)
            .arg("-ss")
            .arg(self.start_time.to_string())
            .arg("-t")
            .arg(self.duration.to_string())
            .arg("-c")
            .arg("copy")
            .arg("-avoid_negative_ts")
            .arg("make_zero")
            .arg("-y")
            .arg(&self.output);
            
        cmd
    }
    
    #[cfg(not(feature = "wasm"))]
    pub fn execute(&self) -> Result<Output> {
        Self::check_ffmpeg_installed()?;
        
        let output = self.build_command()
            .output()
            .map_err(|e| VideoClipError::FFmpegError(e.to_string()))?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VideoClipError::FFmpegError(
                format!("FFmpeg failed: {}", stderr)
            ));
        }
        
        Ok(output)
    }
    
    pub fn get_command_string(&self) -> String {
        format!(
            "ffmpeg -i {} -ss {} -t {} -c copy -avoid_negative_ts make_zero -y {}",
            self.input.display(),
            self.start_time,
            self.duration,
            self.output.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    mod command_building_tests {
        use super::*;
        
        #[test]
        fn test_command_building() {
            let cmd = FFmpegCommand::new(
                "input.mp4",
                "output.mp4",
                10.0,
                30.0
            );
            
            let cmd_string = cmd.get_command_string();
            assert!(cmd_string.contains("-ss 10"));
            assert!(cmd_string.contains("-t 30"));
            assert!(cmd_string.contains("input.mp4"));
            assert!(cmd_string.contains("output.mp4"));
            assert!(cmd_string.contains("-c copy"));
            assert!(cmd_string.contains("-avoid_negative_ts make_zero"));
            assert!(cmd_string.contains("-y"));
        }
        
        #[test]
        fn test_command_with_paths() {
            let temp_dir = tempdir().unwrap();
            let input_path = temp_dir.path().join("test input.mp4");
            let output_path = temp_dir.path().join("test output.mp4");
            
            let cmd = FFmpegCommand::new(
                &input_path,
                &output_path,
                5.5,
                25.75
            );
            
            let cmd_string = cmd.get_command_string();
            assert!(cmd_string.contains("-ss 5.5"));
            assert!(cmd_string.contains("-t 25.75"));
            assert!(cmd_string.contains("test input.mp4"));
            assert!(cmd_string.contains("test output.mp4"));
        }
        
        #[test]
        fn test_command_with_zero_start_time() {
            let cmd = FFmpegCommand::new(
                "input.mp4",
                "output.mp4",
                0.0,
                60.0
            );
            
            let cmd_string = cmd.get_command_string();
            assert!(cmd_string.contains("-ss 0"));
            assert!(cmd_string.contains("-t 60"));
        }
        
        #[test]
        fn test_build_command_args() {
            let cmd = FFmpegCommand::new(
                "input.mp4",
                "output.mp4",
                15.0,
                45.0
            );
            
            let command = cmd.build_command();
            let args: Vec<String> = command.get_args()
                .map(|arg| arg.to_string_lossy().to_string())
                .collect();
            
            assert_eq!(args[0], "-i");
            assert!(args.contains(&"input.mp4".to_string()));
            assert!(args.contains(&"-ss".to_string()));
            assert!(args.contains(&"15".to_string()));
            assert!(args.contains(&"-t".to_string()));
            assert!(args.contains(&"45".to_string()));
            assert!(args.contains(&"-c".to_string()));
            assert!(args.contains(&"copy".to_string()));
            assert!(args.contains(&"-avoid_negative_ts".to_string()));
            assert!(args.contains(&"make_zero".to_string()));
            assert!(args.contains(&"-y".to_string()));
            assert!(args.contains(&"output.mp4".to_string()));
        }
    }
    
    mod ffmpeg_detection_tests {
        use super::*;
        
        #[test]
        fn test_ffmpeg_version_check() {
            // This test will fail if FFmpeg is not installed, which is expected
            // in environments where FFmpeg is not available
            let result = FFmpegCommand::check_ffmpeg_installed();
            // We can't assert success/failure since it depends on system setup
            // But we can verify the error type when it fails
            if result.is_err() {
                match result.unwrap_err() {
                    crate::VideoClipError::FFmpegNotFound => {},
                    _ => panic!("Expected FFmpegNotFound error"),
                }
            }
        }
    }
    
    #[cfg(not(feature = "wasm"))]
    mod execution_tests {
        use super::*;
        use std::io::Write;
        
        #[test]
        fn test_execute_with_missing_input() {
            let cmd = FFmpegCommand::new(
                "nonexistent.mp4",
                "output.mp4",
                0.0,
                10.0
            );
            
            let result = cmd.execute();
            // This should fail due to missing input file (if FFmpeg is available)
            // or due to FFmpeg not being installed
            assert!(result.is_err());
        }
        
        #[test]
        fn test_execute_with_invalid_duration() {
            let temp_dir = tempdir().unwrap();
            let input_path = temp_dir.path().join("test.mp4");
            let output_path = temp_dir.path().join("output.mp4");
            
            // Create a dummy file (not a real video)
            File::create(&input_path).unwrap();
            
            let cmd = FFmpegCommand::new(
                &input_path,
                &output_path,
                0.0,
                -10.0  // Invalid negative duration
            );
            
            let result = cmd.execute();
            // This should fail due to invalid parameters (if FFmpeg is available)
            assert!(result.is_err());
        }
    }
    
    mod edge_case_tests {
        use super::*;
        
        #[test]
        fn test_very_small_duration() {
            let cmd = FFmpegCommand::new(
                "input.mp4",
                "output.mp4",
                100.0,
                0.001  // Very small duration
            );
            
            let cmd_string = cmd.get_command_string();
            assert!(cmd_string.contains("-ss 100"));
            assert!(cmd_string.contains("-t 0.001"));
        }
        
        #[test]
        fn test_large_timestamps() {
            let cmd = FFmpegCommand::new(
                "input.mp4",
                "output.mp4",
                3661.5,  // > 1 hour
                7200.0   // 2 hours
            );
            
            let cmd_string = cmd.get_command_string();
            assert!(cmd_string.contains("-ss 3661.5"));
            assert!(cmd_string.contains("-t 7200"));
        }
        
        #[test]
        fn test_special_characters_in_paths() {
            let cmd = FFmpegCommand::new(
                "my video (2023) - final.mp4",
                "output [clipped].mp4",
                30.0,
                60.0
            );
            
            let cmd_string = cmd.get_command_string();
            assert!(cmd_string.contains("my video (2023) - final.mp4"));
            assert!(cmd_string.contains("output [clipped].mp4"));
        }
    }
    
    mod constructor_tests {
        use super::*;
        
        #[test]
        fn test_new_with_string_paths() {
            let cmd = FFmpegCommand::new(
                "input.mp4",
                "output.mp4",
                10.0,
                20.0
            );
            
            assert_eq!(cmd.input, Path::new("input.mp4"));
            assert_eq!(cmd.output, Path::new("output.mp4"));
            assert_eq!(cmd.start_time, 10.0);
            assert_eq!(cmd.duration, 20.0);
        }
        
        #[test]
        fn test_new_with_path_bufs() {
            let input_path = PathBuf::from("input.mp4");
            let output_path = PathBuf::from("output.mp4");
            
            let cmd = FFmpegCommand::new(
                &input_path,
                &output_path,
                5.5,
                15.25
            );
            
            assert_eq!(cmd.input, input_path);
            assert_eq!(cmd.output, output_path);
            assert_eq!(cmd.start_time, 5.5);
            assert_eq!(cmd.duration, 15.25);
        }
    }
}