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
    audio_codec: AudioCodec,
    preserve_audio_quality: bool,
}

#[derive(Debug, Clone)]
pub enum AudioCodec {
    Copy,
    Aac,
    Mp3,
    Auto,
}

impl FFmpegCommand {
    pub fn new(input: impl AsRef<Path>, output: impl AsRef<Path>, start_time: f64, duration: f64) -> Self {
        Self {
            input: input.as_ref().to_path_buf(),
            output: output.as_ref().to_path_buf(),
            start_time,
            duration,
            audio_codec: AudioCodec::Auto,
            preserve_audio_quality: true,
        }
    }

    pub fn with_audio_options(input: impl AsRef<Path>, output: impl AsRef<Path>, start_time: f64, duration: f64, audio_codec: AudioCodec, preserve_quality: bool) -> Self {
        Self {
            input: input.as_ref().to_path_buf(),
            output: output.as_ref().to_path_buf(),
            start_time,
            duration,
            audio_codec,
            preserve_audio_quality: preserve_quality,
        }
    }

    pub fn set_audio_codec(&mut self, codec: AudioCodec) {
        self.audio_codec = codec;
    }

    pub fn set_preserve_audio_quality(&mut self, preserve: bool) {
        self.preserve_audio_quality = preserve;
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

        // Input and timing
        cmd.arg("-i")
            .arg(&self.input)
            .arg("-ss")
            .arg(self.start_time.to_string())
            .arg("-t")
            .arg(self.duration.to_string());

        // Explicit stream mapping to ensure both video and audio are included
        cmd.arg("-map").arg("0:v?")  // Map video stream if present (? makes it optional)
            .arg("-map").arg("0:a?"); // Map audio stream if present (? makes it optional)

        // Video codec (copy for speed)
        cmd.arg("-c:v").arg("copy");

        // Audio codec handling
        match &self.audio_codec {
            AudioCodec::Copy => {
                cmd.arg("-c:a").arg("copy");
            },
            AudioCodec::Aac => {
                cmd.arg("-c:a").arg("aac");
                if self.preserve_audio_quality {
                    cmd.arg("-b:a").arg("128k"); // Good quality bitrate
                }
            },
            AudioCodec::Mp3 => {
                cmd.arg("-c:a").arg("mp3");
                if self.preserve_audio_quality {
                    cmd.arg("-b:a").arg("128k");
                }
            },
            AudioCodec::Auto => {
                // Try copy first, fallback to AAC if needed
                cmd.arg("-c:a").arg("copy");
            }
        }

        // Audio sync and quality preservation
        cmd.arg("-avoid_negative_ts").arg("make_zero")
            .arg("-async").arg("1") // Audio sync adjustment
            .arg("-vsync").arg("2"); // Video sync for better compatibility

        // Output options
        cmd.arg("-y") // Overwrite output file
            .arg(&self.output);

        cmd
    }
    
    #[cfg(not(feature = "wasm"))]
    pub fn execute(&self) -> Result<Output> {
        Self::check_ffmpeg_installed()?;

        // Try the primary command first
        let output = self.build_command()
            .output()
            .map_err(|e| VideoClipError::FFmpegError(e.to_string()))?;

        if output.status.success() {
            return Ok(output);
        }

        // Check if the error is audio-related and try fallback
        let stderr = String::from_utf8_lossy(&output.stderr);
        if self.is_audio_error(&stderr) {
            println!("Audio copy failed, attempting fallback with AAC encoding...");

            let fallback_output = self.build_fallback_command()
                .output()
                .map_err(|e| VideoClipError::FFmpegError(e.to_string()))?;

            if !fallback_output.status.success() {
                let fallback_stderr = String::from_utf8_lossy(&fallback_output.stderr);
                return Err(VideoClipError::FFmpegError(
                    format!("FFmpeg failed even with fallback: {}", fallback_stderr)
                ));
            }

            return Ok(fallback_output);
        }

        // Non-audio related error, return original error
        Err(VideoClipError::FFmpegError(
            format!("FFmpeg failed: {}", stderr)
        ))
    }

    fn is_audio_error(&self, stderr: &str) -> bool {
        let audio_error_indicators = [
            "codec not currently supported in container",
            "could not find codec parameters for stream",
            "invalid codec tag",
            "audio codec",
            "stream copy",
            "does not support codec",
        ];

        audio_error_indicators.iter().any(|&indicator| {
            stderr.to_lowercase().contains(&indicator.to_lowercase())
        })
    }
    
    pub fn get_command_string(&self) -> String {
        let audio_codec_str = match &self.audio_codec {
            AudioCodec::Copy => "-c:a copy".to_string(),
            AudioCodec::Aac => {
                if self.preserve_audio_quality {
                    "-c:a aac -b:a 128k".to_string()
                } else {
                    "-c:a aac".to_string()
                }
            },
            AudioCodec::Mp3 => {
                if self.preserve_audio_quality {
                    "-c:a mp3 -b:a 128k".to_string()
                } else {
                    "-c:a mp3".to_string()
                }
            },
            AudioCodec::Auto => "-c:a copy".to_string(),
        };

        format!(
            "ffmpeg -i {} -ss {} -t {} -map 0:v? -map 0:a? -c:v copy {} -avoid_negative_ts make_zero -async 1 -vsync 2 -y {}",
            self.input.display(),
            self.start_time,
            self.duration,
            audio_codec_str,
            self.output.display()
        )
    }

    pub fn build_fallback_command(&self) -> Command {
        let mut cmd = Command::new("ffmpeg");

        // Input and timing
        cmd.arg("-i")
            .arg(&self.input)
            .arg("-ss")
            .arg(self.start_time.to_string())
            .arg("-t")
            .arg(self.duration.to_string());

        // Explicit stream mapping
        cmd.arg("-map").arg("0:v?")
            .arg("-map").arg("0:a?");

        // Use AAC as fallback for audio compatibility
        cmd.arg("-c:v").arg("copy")
            .arg("-c:a").arg("aac")
            .arg("-b:a").arg("128k");

        // Audio sync and quality preservation
        cmd.arg("-avoid_negative_ts").arg("make_zero")
            .arg("-async").arg("1")
            .arg("-vsync").arg("2")
            .arg("-y")
            .arg(&self.output);

        cmd
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
            assert!(cmd_string.contains("-c:v copy"));
            assert!(cmd_string.contains("-c:a copy"));
            assert!(cmd_string.contains("-map 0:v?"));
            assert!(cmd_string.contains("-map 0:a?"));
            assert!(cmd_string.contains("-avoid_negative_ts make_zero"));
            assert!(cmd_string.contains("-async 1"));
            assert!(cmd_string.contains("-vsync 2"));
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
            assert!(args.contains(&"-map".to_string()));
            assert!(args.contains(&"0:v?".to_string()));
            assert!(args.contains(&"0:a?".to_string()));
            assert!(args.contains(&"-c:v".to_string()));
            assert!(args.contains(&"-c:a".to_string()));
            assert!(args.contains(&"copy".to_string()));
            assert!(args.contains(&"-avoid_negative_ts".to_string()));
            assert!(args.contains(&"make_zero".to_string()));
            assert!(args.contains(&"-async".to_string()));
            assert!(args.contains(&"1".to_string()));
            assert!(args.contains(&"-vsync".to_string()));
            assert!(args.contains(&"2".to_string()));
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
            std::fs::File::create(&input_path).unwrap();

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
    
    mod audio_handling_tests {
        use super::*;

        #[test]
        fn test_audio_codec_copy() {
            let cmd = FFmpegCommand::with_audio_options(
                "input.mp4",
                "output.mp4",
                10.0,
                30.0,
                AudioCodec::Copy,
                true
            );

            let cmd_string = cmd.get_command_string();
            assert!(cmd_string.contains("-c:a copy"));
            assert!(cmd_string.contains("-map 0:v?"));
            assert!(cmd_string.contains("-map 0:a?"));
            assert!(cmd_string.contains("-async 1"));
            assert!(cmd_string.contains("-vsync 2"));
        }

        #[test]
        fn test_audio_codec_aac_with_quality() {
            let cmd = FFmpegCommand::with_audio_options(
                "input.mp4",
                "output.mp4",
                10.0,
                30.0,
                AudioCodec::Aac,
                true
            );

            let cmd_string = cmd.get_command_string();
            assert!(cmd_string.contains("-c:a aac"));
            assert!(cmd_string.contains("-b:a 128k"));
        }

        #[test]
        fn test_audio_codec_aac_without_quality() {
            let cmd = FFmpegCommand::with_audio_options(
                "input.mp4",
                "output.mp4",
                10.0,
                30.0,
                AudioCodec::Aac,
                false
            );

            let cmd_string = cmd.get_command_string();
            assert!(cmd_string.contains("-c:a aac"));
            assert!(!cmd_string.contains("-b:a 128k"));
        }

        #[test]
        fn test_audio_codec_mp3() {
            let cmd = FFmpegCommand::with_audio_options(
                "input.mp4",
                "output.mp4",
                10.0,
                30.0,
                AudioCodec::Mp3,
                true
            );

            let cmd_string = cmd.get_command_string();
            assert!(cmd_string.contains("-c:a mp3"));
            assert!(cmd_string.contains("-b:a 128k"));
        }

        #[test]
        fn test_audio_codec_auto() {
            let cmd = FFmpegCommand::with_audio_options(
                "input.mp4",
                "output.mp4",
                10.0,
                30.0,
                AudioCodec::Auto,
                true
            );

            let cmd_string = cmd.get_command_string();
            assert!(cmd_string.contains("-c:a copy"));
        }

        #[test]
        fn test_fallback_command_building() {
            let cmd = FFmpegCommand::new(
                "input.mp4",
                "output.mp4",
                15.0,
                45.0
            );

            let fallback_command = cmd.build_fallback_command();
            let args: Vec<String> = fallback_command.get_args()
                .map(|arg| arg.to_string_lossy().to_string())
                .collect();

            assert!(args.contains(&"-c:a".to_string()));
            assert!(args.contains(&"aac".to_string()));
            assert!(args.contains(&"-b:a".to_string()));
            assert!(args.contains(&"128k".to_string()));
            assert!(args.contains(&"-map".to_string()));
            assert!(args.contains(&"0:v?".to_string()));
            assert!(args.contains(&"0:a?".to_string()));
        }

        #[test]
        fn test_audio_error_detection() {
            let cmd = FFmpegCommand::new("input.mp4", "output.mp4", 0.0, 10.0);

            assert!(cmd.is_audio_error("codec not currently supported in container"));
            assert!(cmd.is_audio_error("Could not find codec parameters for stream"));
            assert!(cmd.is_audio_error("Invalid codec tag"));
            assert!(cmd.is_audio_error("Audio codec error occurred"));
            assert!(cmd.is_audio_error("Stream copy failed"));
            assert!(cmd.is_audio_error("Container does not support codec"));

            assert!(!cmd.is_audio_error("File not found"));
            assert!(!cmd.is_audio_error("Permission denied"));
            assert!(!cmd.is_audio_error("Network error"));
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