use crate::error::{VideoClipError, Result};

/// TimeParser provides utilities for parsing and formatting time values
/// Used in video clipping operations to handle start/end times
/// Supports multiple formats: seconds ("120"), MM:SS ("2:30"), HH:MM:SS ("1:30:45")

#[derive(Debug, Clone)]
pub struct TimeParser;

impl TimeParser {
    fn parse_complex_format(time_str: &str) -> Result<f64> {
        let mut total_seconds = 0.0;
        let mut current_number = String::new();
        
        for ch in time_str.chars() {
            if ch.is_ascii_digit() || ch == '.' {
                current_number.push(ch);
            } else if ch == 'h' {
                let hours = current_number.parse::<f64>()
                    .map_err(|_| VideoClipError::InvalidTimeFormat(time_str.to_string()))?;
                total_seconds += hours * 3600.0;
                current_number.clear();
            } else if ch == 'm' {
                let minutes = current_number.parse::<f64>()
                    .map_err(|_| VideoClipError::InvalidTimeFormat(time_str.to_string()))?;
                total_seconds += minutes * 60.0;
                current_number.clear();
            } else if ch == 's' {
                let seconds = current_number.parse::<f64>()
                    .map_err(|_| VideoClipError::InvalidTimeFormat(time_str.to_string()))?;
                total_seconds += seconds;
                current_number.clear();
            } else if !ch.is_whitespace() {
                return Err(VideoClipError::InvalidTimeFormat(time_str.to_string()));
            }
        }
        
        // If there's a remaining number without suffix, treat as seconds
        if !current_number.is_empty() {
            let seconds = current_number.parse::<f64>()
                .map_err(|_| VideoClipError::InvalidTimeFormat(time_str.to_string()))?;
            total_seconds += seconds;
        }
        
        Ok(total_seconds)
    }

    pub fn parse_to_seconds(time_str: &str) -> Result<f64> {
        if time_str.is_empty() {
            return Ok(0.0);
        }
        
        let time_str = time_str.trim();
        
        if time_str.is_empty() {
            return Ok(0.0);
        }
        
        // Fast path for simple numeric values (most common case)
        if time_str.chars().all(|c| c.is_ascii_digit() || c == '.') {
            return time_str.parse::<f64>()
                .map_err(|_| VideoClipError::InvalidTimeFormat(time_str.to_string()));
        }
        
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
            // Check for complex formats first (like "1h30m45s", "2m30s", etc.)
            if time_str.contains('m') || time_str.contains('h') || time_str.contains('s') {
                // Count how many different units we have
                let has_h = time_str.contains('h');
                let has_m = time_str.contains('m');
                let has_s = time_str.contains('s');
                let unit_count = (has_h as u8) + (has_m as u8) + (has_s as u8);
                
                // If more than one unit type, definitely complex format
                if unit_count > 1 {
                    return Self::parse_complex_format(time_str);
                }
                
                // Single unit: check for simple vs complex
                if time_str.ends_with('s') {
                    let num_str = &time_str[..time_str.len()-1];
                    if num_str.chars().all(|c| c.is_ascii_digit() || c == '.') {
                        return num_str.parse::<f64>()
                            .map_err(|_| VideoClipError::InvalidTimeFormat(time_str.to_string()));
                    } else {
                        return Self::parse_complex_format(time_str);
                    }
                } else if time_str.ends_with('m') {
                    let num_str = &time_str[..time_str.len()-1];
                    if num_str.chars().all(|c| c.is_ascii_digit() || c == '.') {
                        let minutes = num_str.parse::<f64>()
                            .map_err(|_| VideoClipError::InvalidTimeFormat(time_str.to_string()))?;
                        return Ok(minutes * 60.0);
                    } else {
                        return Self::parse_complex_format(time_str);
                    }
                } else if time_str.ends_with('h') {
                    let num_str = &time_str[..time_str.len()-1];
                    if num_str.chars().all(|c| c.is_ascii_digit() || c == '.') {
                        let hours = num_str.parse::<f64>()
                            .map_err(|_| VideoClipError::InvalidTimeFormat(time_str.to_string()))?;
                        return Ok(hours * 3600.0);
                    } else {
                        return Self::parse_complex_format(time_str);
                    }
                } else {
                    // Contains units but doesn't end with one, must be complex
                    return Self::parse_complex_format(time_str);
                }
            } else {
                // Try to parse as seconds
                time_str.parse::<f64>()
                    .map_err(|_| VideoClipError::InvalidTimeFormat(time_str.to_string()))
            }
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
    
    mod parse_time_tests {
        use super::*;
        
        #[test]
        fn test_parse_seconds() {
            assert_eq!(TimeParser::parse_to_seconds("120").unwrap(), 120.0);
            assert_eq!(TimeParser::parse_to_seconds("45.5").unwrap(), 45.5);
            assert_eq!(TimeParser::parse_to_seconds("0").unwrap(), 0.0);
            assert_eq!(TimeParser::parse_to_seconds("3600.5").unwrap(), 3600.5);
        }
        
        #[test]
        fn test_parse_mm_ss() {
            assert_eq!(TimeParser::parse_to_seconds("2:30").unwrap(), 150.0);
            assert_eq!(TimeParser::parse_to_seconds("10:45").unwrap(), 645.0);
            assert_eq!(TimeParser::parse_to_seconds("0:30").unwrap(), 30.0);
            assert_eq!(TimeParser::parse_to_seconds("59:59").unwrap(), 3599.0);
            assert_eq!(TimeParser::parse_to_seconds("1:05.5").unwrap(), 65.5);
        }
        
        #[test]
        fn test_parse_hh_mm_ss() {
            assert_eq!(TimeParser::parse_to_seconds("1:30:45").unwrap(), 5445.0);
            assert_eq!(TimeParser::parse_to_seconds("2:00:00").unwrap(), 7200.0);
            assert_eq!(TimeParser::parse_to_seconds("0:05:30").unwrap(), 330.0);
            assert_eq!(TimeParser::parse_to_seconds("12:34:56.5").unwrap(), 45296.5);
        }
        
        #[test]
        fn test_parse_empty_string() {
            assert_eq!(TimeParser::parse_to_seconds("").unwrap(), 0.0);
            assert_eq!(TimeParser::parse_to_seconds("   ").unwrap(), 0.0);
        }
        
        #[test]
        fn test_parse_invalid_formats() {
            assert!(TimeParser::parse_to_seconds("invalid").is_err());
            assert!(TimeParser::parse_to_seconds("1:2:3:4").is_err());
            assert!(TimeParser::parse_to_seconds(":30").is_err());
            assert!(TimeParser::parse_to_seconds("30:").is_err());
            assert!(TimeParser::parse_to_seconds("abc:30").is_err());
            assert!(TimeParser::parse_to_seconds("30:abc").is_err());
        }
        
        #[test]
        fn test_parse_edge_cases() {
            // Large numbers
            assert_eq!(TimeParser::parse_to_seconds("999:59:59").unwrap(), 3599999.0);
            // Decimals in minutes/hours (should work)
            assert_eq!(TimeParser::parse_to_seconds("1.5:30").unwrap(), 120.0);
        }

        #[test]
        fn test_parse_with_units() {
            assert_eq!(TimeParser::parse_to_seconds("30s").unwrap(), 30.0);
            assert_eq!(TimeParser::parse_to_seconds("5m").unwrap(), 300.0);
            assert_eq!(TimeParser::parse_to_seconds("2h").unwrap(), 7200.0);
            assert_eq!(TimeParser::parse_to_seconds("1.5h").unwrap(), 5400.0);
            assert_eq!(TimeParser::parse_to_seconds("2.5m").unwrap(), 150.0);
            assert_eq!(TimeParser::parse_to_seconds("45.7s").unwrap(), 45.7);
        }

        #[test]
        fn test_parse_complex_formats() {
            assert_eq!(TimeParser::parse_to_seconds("1h30m").unwrap(), 5400.0);
            assert_eq!(TimeParser::parse_to_seconds("1h30m45s").unwrap(), 5445.0);
            assert_eq!(TimeParser::parse_to_seconds("2m30s").unwrap(), 150.0);
            assert_eq!(TimeParser::parse_to_seconds("1h45s").unwrap(), 3645.0);
            assert_eq!(TimeParser::parse_to_seconds("90m").unwrap(), 5400.0);
            assert_eq!(TimeParser::parse_to_seconds("1h 30m 45s").unwrap(), 5445.0);
        }

        #[test]
        fn test_parse_invalid_complex_formats() {
            assert!(TimeParser::parse_to_seconds("1h30x").is_err());
            assert!(TimeParser::parse_to_seconds("abc30m").is_err());
            // "30m45" should parse as 30 minutes + 45 seconds = 1845 seconds
            assert_eq!(TimeParser::parse_to_seconds("30m45").unwrap(), 1845.0);
            assert!(TimeParser::parse_to_seconds("h30m").is_err());
        }
    }
    
    mod format_time_tests {
        use super::*;
        
        #[test]
        fn test_format_time() {
            assert_eq!(TimeParser::format_time(150.0), "02-30");
            assert_eq!(TimeParser::format_time(3661.0), "61-01");
            assert_eq!(TimeParser::format_time(0.0), "00-00");
            assert_eq!(TimeParser::format_time(59.9), "00-59");
            assert_eq!(TimeParser::format_time(3600.0), "60-00");
        }
        
        #[test]
        fn test_format_time_readable() {
            assert_eq!(TimeParser::format_time_readable(150.0), "02:30");
            assert_eq!(TimeParser::format_time_readable(3661.0), "01:01:01");
            assert_eq!(TimeParser::format_time_readable(0.0), "00:00");
            assert_eq!(TimeParser::format_time_readable(59.0), "00:59");
            assert_eq!(TimeParser::format_time_readable(3600.0), "01:00:00");
            assert_eq!(TimeParser::format_time_readable(7323.5), "02:02:03");
        }
        
        #[test]
        fn test_format_edge_cases() {
            // Very large times
            assert_eq!(TimeParser::format_time_readable(359999.0), "99:59:59");
            // Fractional seconds (should truncate)
            assert_eq!(TimeParser::format_time_readable(61.9), "01:01");
        }
    }
    
    mod validation_tests {
        use super::*;
        
        #[test]
        fn test_validate_time_range() {
            assert!(TimeParser::validate_time_range(100.0, 200.0).is_ok());
            assert_eq!(TimeParser::validate_time_range(100.0, 200.0).unwrap(), 100.0);
            assert!(TimeParser::validate_time_range(0.0, 0.1).is_ok());
            assert_eq!(TimeParser::validate_time_range(0.0, 0.1).unwrap(), 0.1);
        }
        
        #[test]
        fn test_validate_invalid_ranges() {
            assert!(TimeParser::validate_time_range(200.0, 100.0).is_err());
            assert!(TimeParser::validate_time_range(100.0, 100.0).is_err());
            assert!(TimeParser::validate_time_range(-10.0, 50.0).is_ok()); // Negative start is technically valid
        }
        
        #[test]
        fn test_validate_error_messages() {
            match TimeParser::validate_time_range(200.0, 100.0) {
                Err(crate::VideoClipError::InvalidTimeRange { start, end }) => {
                    assert_eq!(start, 200.0);
                    assert_eq!(end, 100.0);
                },
                _ => panic!("Expected InvalidTimeRange error"),
            }
        }
    }
    
    mod integration_tests {
        use super::*;
        
        #[test]
        fn test_parse_and_validate_workflow() {
            let start = TimeParser::parse_to_seconds("1:30").unwrap();
            let end = TimeParser::parse_to_seconds("2:45").unwrap();
            let duration = TimeParser::validate_time_range(start, end).unwrap();
            assert_eq!(duration, 75.0);
        }
        
        #[test]
        fn test_format_after_parse() {
            let time_str = "1:30:45";
            let seconds = TimeParser::parse_to_seconds(time_str).unwrap();
            let formatted = TimeParser::format_time_readable(seconds);
            assert_eq!(formatted, "01:30:45");
        }
    }
}