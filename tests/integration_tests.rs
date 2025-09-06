use video_clip_rs::{TimeParser, VideoClipper, VideoClipError};
use video_clip_rs::video_clipper::{ClipRequest, ClipResult};

mod time_parsing_tests {
    use super::*;

    #[test]
    fn test_all_time_formats() {
        // Test basic seconds
        assert_eq!(TimeParser::parse_to_seconds("30").unwrap(), 30.0);
        assert_eq!(TimeParser::parse_to_seconds("90.5").unwrap(), 90.5);
        
        // Test MM:SS format
        assert_eq!(TimeParser::parse_to_seconds("1:30").unwrap(), 90.0);
        assert_eq!(TimeParser::parse_to_seconds("10:45").unwrap(), 645.0);
        
        // Test HH:MM:SS format
        assert_eq!(TimeParser::parse_to_seconds("1:05:30").unwrap(), 3930.0);
        assert_eq!(TimeParser::parse_to_seconds("2:00:15").unwrap(), 7215.0);
        
        // Test unit suffixes
        assert_eq!(TimeParser::parse_to_seconds("45s").unwrap(), 45.0);
        assert_eq!(TimeParser::parse_to_seconds("5m").unwrap(), 300.0);
        assert_eq!(TimeParser::parse_to_seconds("2h").unwrap(), 7200.0);
        
        // Test complex formats
        assert_eq!(TimeParser::parse_to_seconds("1h30m").unwrap(), 5400.0);
        assert_eq!(TimeParser::parse_to_seconds("2h15m30s").unwrap(), 8130.0);
        assert_eq!(TimeParser::parse_to_seconds("1m30s").unwrap(), 90.0);
    }

    #[test]
    fn test_invalid_formats() {
        assert!(TimeParser::parse_to_seconds("invalid").is_err());
        assert!(TimeParser::parse_to_seconds("1:2:3:4").is_err());
        assert!(TimeParser::parse_to_seconds("abc:30").is_err());
        assert!(TimeParser::parse_to_seconds("30:abc").is_err());
        assert!(TimeParser::parse_to_seconds("1h30x").is_err());
    }

    #[test]
    fn test_time_range_validation() {
        // Valid ranges
        assert_eq!(TimeParser::validate_time_range(0.0, 30.0).unwrap(), 30.0);
        assert_eq!(TimeParser::validate_time_range(60.0, 120.0).unwrap(), 60.0);
        
        // Invalid ranges
        assert!(TimeParser::validate_time_range(120.0, 60.0).is_err());
        assert!(TimeParser::validate_time_range(100.0, 100.0).is_err());
        
        // Check error type
        match TimeParser::validate_time_range(100.0, 50.0) {
            Err(VideoClipError::InvalidTimeRange { start, end }) => {
                assert_eq!(start, 100.0);
                assert_eq!(end, 50.0);
            }
            _ => panic!("Expected InvalidTimeRange error"),
        }
    }

    #[test]
    fn test_time_formatting() {
        // Test readable formatting
        assert_eq!(TimeParser::format_time_readable(90.0), "01:30");
        assert_eq!(TimeParser::format_time_readable(3661.0), "01:01:01");
        assert_eq!(TimeParser::format_time_readable(0.0), "00:00");
        assert_eq!(TimeParser::format_time_readable(7320.0), "02:02:00");
        
        // Test filename-safe formatting
        assert_eq!(TimeParser::format_time(90.0), "01-30");
        assert_eq!(TimeParser::format_time(3661.0), "61-01");
    }
}

mod video_clipper_tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;

    #[test]
    fn test_clipper_creation() {
        let clipper = VideoClipper::new();
        assert!(format!("{:?}", clipper).contains("VideoClipper"));
        
        let temp_dir = tempdir().unwrap();
        let clipper = VideoClipper::with_output_dir(temp_dir.path());
        assert!(format!("{:?}", clipper).contains("VideoClipper"));
    }

    #[test]
    fn test_filename_generation() {
        let clipper = VideoClipper::new();
        let input_path = std::path::Path::new("test_video.mp4");
        
        let output = clipper.generate_output_filename(input_path, 30.0, 120.0);
        let filename = output.file_name().unwrap().to_string_lossy();
        
        assert!(filename.contains("test_video_clip_00-30_to_02-00.mp4"));
        assert!(filename.ends_with(".mp4"));
    }

    #[test]
    fn test_filename_with_special_characters() {
        let clipper = VideoClipper::new();
        let input_path = std::path::Path::new("My Video (2023).mov");
        
        let output = clipper.generate_output_filename(input_path, 0.0, 60.0);
        let filename = output.file_name().unwrap().to_string_lossy();
        
        assert!(filename.contains("My Video (2023)_clip"));
        assert!(filename.ends_with(".mp4"));
    }

    #[test]
    fn test_ensure_output_dir() {
        let temp_dir = tempdir().unwrap();
        let nested_path = temp_dir.path().join("nested").join("output");
        let clipper = VideoClipper::with_output_dir(&nested_path);
        
        assert!(!nested_path.exists());
        clipper.ensure_output_dir().unwrap();
        assert!(nested_path.exists());
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn test_prepare_clip_command() {
        let clipper = VideoClipper::new();
        let request = ClipRequest {
            input_file: "test.mp4".to_string(),
            start_time: "30s".to_string(),
            end_time: "2m".to_string(),
            output_dir: None,
        };

        let result = clipper.prepare_clip_command(&request);
        assert!(result.is_ok());

        let clip_result = result.unwrap();
        assert_eq!(clip_result.start_seconds, 30.0);
        assert_eq!(clip_result.end_seconds, 120.0);
        assert_eq!(clip_result.duration, 90.0);
        assert!(clip_result.command.contains("ffmpeg"));
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn test_prepare_clip_invalid_times() {
        let clipper = VideoClipper::new();
        let request = ClipRequest {
            input_file: "test.mp4".to_string(),
            start_time: "2m".to_string(),
            end_time: "1m".to_string(), // End before start
            output_dir: None,
        };

        let result = clipper.prepare_clip_command(&request);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            VideoClipError::InvalidTimeRange { .. } => {}
            _ => panic!("Expected InvalidTimeRange error"),
        }
    }
}

mod serialization_tests {
    use super::*;

    #[test]
    fn test_clip_request_serialization() {
        let request = ClipRequest {
            input_file: "video.mp4".to_string(),
            start_time: "1m30s".to_string(),
            end_time: "3m45s".to_string(),
            output_dir: Some("/tmp/clips".to_string()),
        };

        // Test JSON serialization
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
            input_file: "input.mp4".to_string(),
            output_file: "output_clip_01-30_to_03-45.mp4".to_string(),
            start_seconds: 90.0,
            end_seconds: 225.0,
            duration: 135.0,
            file_size_mb: Some(12.5),
            command: "ffmpeg -i input.mp4 -ss 90 -t 135 -c copy output.mp4".to_string(),
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ClipResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.start_seconds, deserialized.start_seconds);
        assert_eq!(result.end_seconds, deserialized.end_seconds);
        assert_eq!(result.duration, deserialized.duration);
        assert_eq!(result.file_size_mb, deserialized.file_size_mb);
        assert_eq!(result.command, deserialized.command);
    }
}

mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_time_parsing_performance() {
        let test_times = vec![
            "30", "1:30", "2:45", "1:15:30", "45s", "5m", "2h", 
            "1h30m", "2h15m30s", "90m", "3600s", "0:05", "10:00:00"
        ];

        let start = Instant::now();
        
        for _ in 0..1000 {
            for time_str in &test_times {
                TimeParser::parse_to_seconds(time_str).unwrap();
            }
        }
        
        let elapsed = start.elapsed();
        println!("Parsed {} time strings in {:?}", test_times.len() * 1000, elapsed);
        
        // Should be able to parse thousands of time strings per millisecond
        assert!(elapsed.as_millis() < 100);
    }

    #[test]
    fn test_filename_generation_performance() {
        let clipper = VideoClipper::new();
        let input_path = std::path::Path::new("test_video_with_long_name.mp4");
        
        let start = Instant::now();
        
        for i in 0..1000 {
            let start_time = i as f64;
            let end_time = start_time + 60.0;
            clipper.generate_output_filename(input_path, start_time, end_time);
        }
        
        let elapsed = start.elapsed();
        println!("Generated 1000 filenames in {:?}", elapsed);
        
        // Should be able to generate thousands of filenames per millisecond
        assert!(elapsed.as_millis() < 50);
    }

    #[test] 
    fn test_validation_performance() {
        let start = Instant::now();
        
        for i in 0..10000 {
            let start_time = (i as f64) * 0.1;
            let end_time = start_time + 30.0;
            TimeParser::validate_time_range(start_time, end_time).unwrap();
        }
        
        let elapsed = start.elapsed();
        println!("Validated 10000 time ranges in {:?}", elapsed);
        
        // Should be extremely fast for simple numeric validation
        assert!(elapsed.as_millis() < 10);
    }
}

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_very_long_durations() {
        // Test 24+ hour videos
        assert_eq!(TimeParser::parse_to_seconds("25:30:45").unwrap(), 91845.0);
        assert_eq!(TimeParser::parse_to_seconds("100:00:00").unwrap(), 360000.0);
        
        // Test complex long formats
        assert_eq!(TimeParser::parse_to_seconds("48h30m15s").unwrap(), 174615.0);
    }

    #[test]
    fn test_decimal_precision() {
        // Test precise decimal handling
        assert_eq!(TimeParser::parse_to_seconds("30.123").unwrap(), 30.123);
        assert_eq!(TimeParser::parse_to_seconds("1:30.456").unwrap(), 90.456);
        assert_eq!(TimeParser::parse_to_seconds("2.5m").unwrap(), 150.0);
    }

    #[test]
    fn test_whitespace_handling() {
        // Test various whitespace scenarios
        assert_eq!(TimeParser::parse_to_seconds(" 30 ").unwrap(), 30.0);
        assert_eq!(TimeParser::parse_to_seconds("1h 30m 45s").unwrap(), 5445.0);
        assert_eq!(TimeParser::parse_to_seconds("  1:30:45  ").unwrap(), 5445.0);
    }

    #[test]
    fn test_zero_duration_edge_cases() {
        // Test zero values
        assert_eq!(TimeParser::parse_to_seconds("0").unwrap(), 0.0);
        assert_eq!(TimeParser::parse_to_seconds("0:00").unwrap(), 0.0);
        assert_eq!(TimeParser::parse_to_seconds("0h0m0s").unwrap(), 0.0);
        
        // Test very small durations
        assert_eq!(TimeParser::validate_time_range(0.0, 0.1).unwrap(), 0.1);
        assert_eq!(TimeParser::validate_time_range(100.0, 100.001).unwrap(), 0.001);
    }

    #[test]
    fn test_filename_edge_cases() {
        let clipper = VideoClipper::new();
        
        // Empty filename
        let output = clipper.generate_output_filename(std::path::Path::new(""), 0.0, 30.0);
        let filename = output.file_name().unwrap().to_string_lossy();
        assert!(filename.contains("clip_00-00_to_00-30.mp4"));
        
        // Very long filename (should be handled gracefully)
        let long_name = "a".repeat(200) + ".mp4";
        let output = clipper.generate_output_filename(std::path::Path::new(&long_name), 0.0, 30.0);
        assert!(output.file_name().is_some());
    }
}

mod error_handling_tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        // Test InvalidTimeFormat error
        match TimeParser::parse_to_seconds("invalid_time") {
            Err(VideoClipError::InvalidTimeFormat(msg)) => {
                assert!(msg.contains("invalid_time"));
            }
            _ => panic!("Expected InvalidTimeFormat error"),
        }

        // Test InvalidTimeRange error
        match TimeParser::validate_time_range(100.0, 50.0) {
            Err(VideoClipError::InvalidTimeRange { start, end }) => {
                assert_eq!(start, 100.0);
                assert_eq!(end, 50.0);
            }
            _ => panic!("Expected InvalidTimeRange error"),
        }
    }

    #[test]
    fn test_error_display() {
        let error = VideoClipError::InvalidTimeFormat("bad_time".to_string());
        let error_str = format!("{}", error);
        assert!(error_str.contains("Invalid time format: bad_time"));

        let error = VideoClipError::InvalidTimeRange { start: 100.0, end: 50.0 };
        let error_str = format!("{}", error);
        assert!(error_str.contains("End time (50) must be after start time (100)"));
    }
}