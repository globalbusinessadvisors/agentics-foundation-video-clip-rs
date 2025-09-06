use crate::error::{VideoClipError, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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
    }
}