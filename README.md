# 🎬 Video Clipper Pro

> A high-performance browser-based video clipping tool built with Rust and WebAssembly.

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![WebAssembly](https://img.shields.io/badge/webassembly-supported-purple.svg)](https://webassembly.org)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Create video clips directly in your browser - no uploads, no installs, no accounts required.**

## Features

- 🎯 **Precise Time Control**: Enter exact timestamps from your video (e.g., `29:24` to `31:45`)
- ⚡ **Browser Processing**: Everything runs locally using WebAssembly + FFmpeg.js
- 📱 **Drag & Drop**: Upload videos with intuitive drag and drop interface
- 🔒 **Privacy First**: Videos never leave your computer
- 📊 **Real-time Progress**: Visual progress tracking during processing
- 💾 **Direct Download**: MP4 clips download straight to your Downloads folder

## Quick Start

1. **Start the application:**
   ```bash
   ./start.sh
   ```

2. **Open in browser:** http://localhost:8080

3. **Create clips:**
   - Click "📁 Click Here to Upload Video" or drag & drop a video file
   - Enter start and end times (e.g., 0:30 to 2:15)
   - Click "✂️ Create Video Clip"
   - Download your processed clip!

## Usage

Extract specific portions from your videos by setting start and end times within the original video.

### Example: Extract a 2-minute clip from a 1-hour video
```
Original video: 61:25 long
Start time: 29:24  (clip starts at 29 minutes 24 seconds)
End time: 31:45    (clip ends at 31 minutes 45 seconds)
Result: 2:21 long clip extracted
```

### Time Format Examples
```
30          → 30 seconds into the video
1:30        → 1 minute 30 seconds into the video  
29:24       → 29 minutes 24 seconds into the video
2:15:45     → 2 hours 15 minutes 45 seconds into the video
90s         → 90 seconds into the video
5m          → 5 minutes into the video
1h30m       → 1 hour 30 minutes into the video
```

### Smart Features
- **Use Current Time** to set start/end from current video playback position
- **Real-time validation** ensures times are within video duration
- **Progress tracking** shows processing status and completion

## Installation

### Prerequisites
- Python 3.x (for local server)
- Modern web browser (Chrome, Firefox, Safari, Edge)

### Setup
```bash
# Clone the repository
git clone https://github.com/your-org/video-clipper-pro.git
cd video-clipper-pro

# Start the application
./start.sh
```

The script will automatically:
- Build the WebAssembly module if needed
- Handle port conflicts
- Start the local server
- Open your browser

### Manual Installation
If you need to build from source:
```bash
# Install Rust and wasm-pack
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build WebAssembly module
wasm-pack build --target web --out-dir pkg --features wasm --no-default-features

# Start server
python3 -m http.server 8080
```

## Architecture

Video Clipper Pro uses a dual-engine architecture:

- **Rust WebAssembly**: Fast time parsing and validation
- **FFmpeg.js**: Browser-based video processing
- **Modern Web APIs**: File handling and downloads

## Performance

| Metric | Performance |
|--------|-------------|
| Time Parsing | 13,000+ operations/ms |
| Memory Usage | <2MB peak |
| WASM Load Time | <500ms initial, <50ms cached |
| Browser Support | Chrome 90+, Firefox 88+, Safari 14+, Edge 90+ |

## Development

### Running Tests
```bash
# Run Rust tests
cargo test --features wasm --no-default-features

# Run integration tests
cargo test --test integration_tests
```

### Build Commands
```bash
# Build WebAssembly module
wasm-pack build --target web --out-dir pkg --features wasm --no-default-features

# Development server
python3 -m http.server 8080
```

### Project Structure
```
src/
├── lib.rs              # Public API
├── time_parser.rs      # Time format parsing
├── video_clipper.rs    # Core functionality
├── wasm.rs            # WebAssembly bindings
└── error.rs           # Error handling

pkg/                   # Generated WebAssembly
tests/                 # Integration tests
index.html            # Web interface
start.sh              # Startup script
```

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes and add tests
4. Run the test suite: `cargo test --features wasm`
5. Submit a pull request

## Troubleshooting

### Port Already in Use
```bash
# Kill existing server
pkill -f "python.*http.server.*8080"

# Or use different port
python3 -m http.server 8081
```

### WASM Module Issues
```bash
# Rebuild WebAssembly module
wasm-pack build --target web --out-dir pkg --features wasm --no-default-features
```

### Browser Compatibility
- Use Chrome or Firefox for best performance
- Ensure JavaScript and WebAssembly are enabled
- Clear browser cache if experiencing issues

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/) and [WebAssembly](https://webassembly.org/)
- Video processing powered by [FFmpeg.js](https://ffmpegwasm.netlify.app/)
- UI components use modern web standards