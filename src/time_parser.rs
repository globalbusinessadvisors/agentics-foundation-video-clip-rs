use crate::error::{VideoClipError, Result};

#[derive(Debug, Clone)]
pub struct TimeParser;

impl TimeParser {
    pub fn parse_to_seconds(time_str: &str) -> Result<f64> {
        if time_str.is_empty() {
            return Ok(0.0);
        }
        
        let time_str = time_str.trim();
        
        // Check if it contains colons (time format)
        if time_str.contains(':') {
            let parts: Vec<&str> = time_str.split(':').collect();
            match parts.len() {
                2 => {
                    // MM:SS format
                    let minutes = parts[0].parse::<f64>()
                        .map_err(|_| VideoClipError::InvalidTimeFormat(time_str.to_string()))?;
                    let seconds = parts[1].parse::<f64>()
                        .map_err(|_| VideoClipError::InvalidTimeFormat(time_str.to_string()))?;
                    Ok(minutes * 60.0 + seconds)
                }
                3 => {
                    // HH:MM:SS format
                    let hours = parts[0].parse::<f64>()
                        .map_err(|_| VideoClipError::InvalidTimeFormat(time_str.to_string()))?;
                    let minutes = parts[1].parse::<f64>()
                        .map_err(|_| VideoClipError::InvalidTimeFormat(time_str.to_string()))?;
                    let seconds = parts[2].parse::<f64>()
                        .map_err(|_| VideoClipError::InvalidTimeFormat(time_str.to_string()))?;
                    Ok(hours * 3600.0 + minutes * 60.0 + seconds)
                }
                _ => Err(VideoClipError::InvalidTimeFormat(time_str.to_string()))
            }
        } else {
            // Try to parse as seconds
            time_str.parse::<f64>()
                .map_err(|_| VideoClipError::InvalidTimeFormat(time_str.to_string()))
        }
    }
    
    pub fn format_time(seconds: f64) -> String {
        let mins = (seconds / 60.0) as u32;
        let secs = (seconds % 60.0) as u32;
        format!("{:02}-{:02}", mins, secs)
    }
    
    pub fn format_time_readable(seconds: f64) -> String {
        let hours = (seconds / 3600.0) as u32;
        let mins = ((seconds % 3600.0) / 60.0) as u32;
        let secs = (seconds % 60.0) as u32;
        
        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, mins, secs)
        } else {
            format!("{:02}:{:02}", mins, secs)
        }
    }
    
    pub fn validate_time_range(start_seconds: f64, end_seconds: f64) -> Result<f64> {
        if end_seconds <= start_seconds {
            return Err(VideoClipError::InvalidTimeRange {
                start: start_seconds,
                end: end_seconds,
            });
        }
        Ok(end_seconds - start_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_seconds() {
        assert_eq!(TimeParser::parse_to_seconds("120").unwrap(), 120.0);
        assert_eq!(TimeParser::parse_to_seconds("45.5").unwrap(), 45.5);
    }
    
    #[test]
    fn test_parse_mm_ss() {
        assert_eq!(TimeParser::parse_to_seconds("2:30").unwrap(), 150.0);
        assert_eq!(TimeParser::parse_to_seconds("10:45").unwrap(), 645.0);
    }
    
    #[test]
    fn test_parse_hh_mm_ss() {
        assert_eq!(TimeParser::parse_to_seconds("1:30:45").unwrap(), 5445.0);
        assert_eq!(TimeParser::parse_to_seconds("2:00:00").unwrap(), 7200.0);
    }
    
    #[test]
    fn test_format_time() {
        assert_eq!(TimeParser::format_time(150.0), "02-30");
        assert_eq!(TimeParser::format_time(3661.0), "61-01");
    }
    
    #[test]
    fn test_format_time_readable() {
        assert_eq!(TimeParser::format_time_readable(150.0), "02:30");
        assert_eq!(TimeParser::format_time_readable(3661.0), "01:01:01");
    }
    
    #[test]
    fn test_validate_time_range() {
        assert!(TimeParser::validate_time_range(100.0, 200.0).is_ok());
        assert!(TimeParser::validate_time_range(200.0, 100.0).is_err());
        assert!(TimeParser::validate_time_range(100.0, 100.0).is_err());
    }
}