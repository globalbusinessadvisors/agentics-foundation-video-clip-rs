#[cfg(feature = "cli")]
use clap::Parser;
#[cfg(feature = "cli")]
use colored::*;
#[cfg(feature = "cli")]
use video_clip_rs::{VideoClipper, ClipRequest, Result};
#[cfg(feature = "cli")]
use std::io::{self, Write};

#[cfg(feature = "cli")]
#[derive(Parser, Debug)]
#[command(author, version, about = "High-performance video clipping tool", long_about = None)]
struct Args {
    /// Input video file path
    #[arg(value_name = "FILE")]
    input: Option<String>,
    
    /// Start time (e.g., 36:07 or 2167)
    #[arg(short, long)]
    start: Option<String>,
    
    /// End time (e.g., 37:19 or 2239)
    #[arg(short, long)]
    end: Option<String>,
    
    /// Output directory (default: downloads)
    #[arg(short, long)]
    output_dir: Option<String>,
}

#[cfg(feature = "cli")]
fn print_banner() {
    println!("{}", "=".repeat(60).bright_blue());
    println!("{} {}", "🎬".bright_yellow(), "VIDEO CLIPPER - Rust Edition".bright_cyan().bold());
    println!("   {}", "High-performance video clipping with WebAssembly support".bright_white());
    println!("{}", "=".repeat(60).bright_blue());
    println!();
}

#[cfg(feature = "cli")]
fn get_input(prompt: &str) -> String {
    print!("{}: ", prompt.bright_yellow());
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().trim_matches('"').trim_matches('\'').to_string()
}

#[cfg(feature = "cli")]
fn main() -> Result<()> {
    env_logger::init();
    
    let args = Args::parse();
    
    print_banner();
    
    // Get input file
    let input_file = match args.input {
        Some(f) => f,
        None => {
            println!("{}", "Enter path to your video file:".bright_cyan());
            println!("{}", "Note: The file must be accessible from this environment".bright_white());
            println!();
            get_input("Video file path")
        }
    };
    
    if input_file.is_empty() {
        eprintln!("{} {}", "❌".bright_red(), "No file path provided!".red());
        std::process::exit(1);
    }
    
    // Get start time
    let start_time = match args.start {
        Some(s) => s,
        None => {
            println!();
            println!("{}", "Start time (e.g., 36:07 or 2167):".bright_cyan());
            let input = get_input("Start");
            if input.is_empty() { "0".to_string() } else { input }
        }
    };
    
    // Get end time
    let end_time = match args.end {
        Some(e) => e,
        None => {
            println!();
            println!("{}", "End time (e.g., 37:19 or 2239):".bright_cyan());
            get_input("End")
        }
    };
    
    if end_time.is_empty() {
        eprintln!("{} {}", "❌".bright_red(), "End time required!".red());
        std::process::exit(1);
    }
    
    // Create clip request
    let request = ClipRequest {
        input_file: input_file.clone(),
        start_time,
        end_time,
        output_dir: args.output_dir,
    };
    
    // Create clipper
    let clipper = VideoClipper::new();
    
    println!();
    println!("{} {}", "✂️".bright_yellow(), "Creating clip:".bright_cyan());
    println!("   {} {}", "Input:".bright_white(), input_file);
    println!("   {} {}", "Start:".bright_white(), request.start_time);
    println!("   {} {}", "End:".bright_white(), request.end_time);
    
    println!();
    println!("{} {}", "⏳".bright_yellow(), "Processing...".bright_cyan());
    
    // Execute clipping
    match clipper.clip_video(&request) {
        Ok(result) => {
            println!();
            println!("{} {}", "✅".bright_green(), "SUCCESS!".bright_green().bold());
            println!("{} {}", "📁 Clip saved:".bright_white(), result.output_file.bright_cyan());
            
            if let Some(size_mb) = result.file_size_mb {
                println!("{} {:.1} MB", "📊 Size:".bright_white(), size_mb);
            }
            
            println!("{} {:.1}s", "⏱️ Duration:".bright_white(), result.duration);
            println!();
            println!("{} {}", "🎉".bright_yellow(), "Done! Your clip is ready!".bright_green().bold());
        }
        Err(e) => {
            eprintln!();
            eprintln!("{} {}", "❌".bright_red(), format!("Error: {}", e).red());
            std::process::exit(1);
        }
    }
    
    Ok(())
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("This binary requires the 'cli' feature to be enabled.");
    eprintln!("Build with: cargo build --features cli");
    std::process::exit(1);
}
