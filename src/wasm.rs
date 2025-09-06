use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::{to_value, from_value};
use crate::{VideoClipper, TimeParser};
use crate::video_clipper::{ClipRequest, ClipResult};

#[wasm_bindgen]
pub fn init_wasm() {
    // Initialize panic hook for better error messages in browser
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
    
    // Initialize logger for WASM
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("Video Clipper WASM initialized");
}

#[wasm_bindgen]
pub fn parse_time_to_seconds(time_str: &str) -> Result<f64, JsValue> {
    TimeParser::parse_to_seconds(time_str)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn format_time(seconds: f64) -> String {
    TimeParser::format_time(seconds)
}

#[wasm_bindgen]
pub fn format_time_readable(seconds: f64) -> String {
    TimeParser::format_time_readable(seconds)
}

#[wasm_bindgen]
pub fn validate_time_range(start_seconds: f64, end_seconds: f64) -> Result<f64, JsValue> {
    TimeParser::validate_time_range(start_seconds, end_seconds)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn generate_ffmpeg_command(
    input_file: &str,
    output_file: &str,
    start_time: &str,
    end_time: &str,
) -> Result<String, JsValue> {
    let start_sec = TimeParser::parse_to_seconds(start_time)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let end_sec = TimeParser::parse_to_seconds(end_time)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let duration = TimeParser::validate_time_range(start_sec, end_sec)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    let command = format!(
        "ffmpeg -i {} -ss {} -t {} -c copy -avoid_negative_ts make_zero -y {}",
        input_file, start_sec, duration, output_file
    );
    
    Ok(command)
}

#[wasm_bindgen]
pub fn prepare_clip(request_js: JsValue) -> Result<JsValue, JsValue> {
    let request: ClipRequest = from_value(request_js)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    let clipper = VideoClipper::new();
    let result = clipper.prepare_clip_command(&request)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    to_value(&result)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub struct WasmVideoClipper {
    clipper: VideoClipper,
}

#[wasm_bindgen]
impl WasmVideoClipper {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        init_wasm();
        Self {
            clipper: VideoClipper::new(),
        }
    }
    
    #[wasm_bindgen]
    pub fn with_output_dir(output_dir: &str) -> Self {
        init_wasm();
        Self {
            clipper: VideoClipper::with_output_dir(output_dir),
        }
    }
    
    #[wasm_bindgen]
    pub fn prepare_clip_command(&self, request_js: JsValue) -> Result<JsValue, JsValue> {
        let request: ClipRequest = from_value(request_js)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        let result = self.clipper.prepare_clip_command(&request)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        to_value(&result)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    
    #[wasm_bindgen]
    pub fn generate_output_filename(&self, input_file: &str, start_time: &str, end_time: &str) -> Result<String, JsValue> {
        let start_sec = TimeParser::parse_to_seconds(start_time)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let end_sec = TimeParser::parse_to_seconds(end_time)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        let path = std::path::Path::new(input_file);
        let output = self.clipper.generate_output_filename(path, start_sec, end_sec);
        
        Ok(output.display().to_string())
    }
}

// Re-export for JavaScript
#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export interface ClipRequest {
    input_file: string;
    start_time: string;
    end_time: string;
    output_dir?: string;
}

export interface ClipResult {
    input_file: string;
    output_file: string;
    start_seconds: number;
    end_seconds: number;
    duration: number;
    file_size_mb?: number;
    command: string;
}
"#;