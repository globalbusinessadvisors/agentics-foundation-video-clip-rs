// Example demonstrating the improved audio handling in video clipper
// This example shows how to use different audio codec options

use video_clip_rs::ffmpeg::{FFmpegCommand, AudioCodec};

fn main() {
    println!("=== Video Clipper Audio Handling Test ===\n");

    // Test 1: Default auto-detection (tries copy first, fallback to AAC)
    let auto_cmd = FFmpegCommand::new(
        "input.mp4",
        "output_auto.mp4",
        0.0,
        30.0
    );
    println!("1. Auto codec detection command:");
    println!("   {}\n", auto_cmd.get_command_string());

    // Test 2: Force AAC encoding for compatibility
    let aac_cmd = FFmpegCommand::with_audio_options(
        "input.mov",
        "output_aac.mp4",
        15.5,
        60.0,
        AudioCodec::Aac,
        true  // preserve quality
    );
    println!("2. Force AAC encoding command:");
    println!("   {}\n", aac_cmd.get_command_string());

    // Test 3: Copy audio stream (fastest)
    let copy_cmd = FFmpegCommand::with_audio_options(
        "input.mkv",
        "output_copy.mp4",
        10.0,
        45.0,
        AudioCodec::Copy,
        true
    );
    println!("3. Copy audio stream command:");
    println!("   {}\n", copy_cmd.get_command_string());

    // Test 4: MP3 audio encoding
    let mp3_cmd = FFmpegCommand::with_audio_options(
        "input.avi",
        "output_mp3.mp4",
        0.0,
        120.0,
        AudioCodec::Mp3,
        false  // don't preserve quality (smaller file)
    );
    println!("4. MP3 audio encoding command:");
    println!("   {}\n", mp3_cmd.get_command_string());

    // Test 5: Fallback command example
    let fallback_cmd = copy_cmd.build_fallback_command();
    let fallback_args: Vec<String> = fallback_cmd.get_args()
        .map(|arg| arg.to_string_lossy().to_string())
        .collect();
    println!("5. Fallback command args: {:?}\n", fallback_args);

    println!("=== Key Audio Improvements ===");
    println!("✓ Explicit stream mapping (-map 0:v? -map 0:a?)");
    println!("✓ Separate video/audio codec control (-c:v copy -c:a aac)");
    println!("✓ Audio sync preservation (-async 1 -vsync 2)");
    println!("✓ Quality preservation options (-b:a 128k)");
    println!("✓ Automatic fallback to AAC if copy fails");
    println!("✓ Audio error detection and recovery");
    println!("✓ Support for various input formats (MP4, MOV, MKV, AVI)");
}