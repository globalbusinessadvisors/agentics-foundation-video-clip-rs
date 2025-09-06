# Browser-Based Video Clipper Guide

## Overview

This solution provides a complete browser-based video clipping tool that:
- Uploads videos directly in the browser
- Processes them using WebAssembly (FFmpeg.wasm + Rust WASM)
- Downloads the clipped result as MP4
- **Never sends video data to any server** - everything runs locally

## Features

### 🔒 Complete Privacy
- Videos are processed entirely in your browser
- No server uploads required
- No data leaves your device

### ⚡ High Performance
- Rust WASM for time parsing and command generation
- FFmpeg.wasm for actual video processing
- Optimized for speed and efficiency

### 🎯 User-Friendly Interface
- Drag & drop video upload
- Visual timeline with preview
- Click to set start/end from current playback time
- Real-time progress indication
- One-click download of results

## How to Use

### Quick Start

1. **Start the web server:**
   ```bash
   python3 -m http.server 8000
   ```

2. **Open in browser:**
   ```
   http://localhost:8000/index_advanced.html
   ```

3. **Upload your video:**
   - Click the upload area or drag & drop
   - Supports MP4, WebM, MOV, and most video formats

4. **Set clip times:**
   - Enter times manually (format: MM:SS or HH:MM:SS)
   - OR click "Use Current Time" buttons while previewing

5. **Create clip:**
   - Click "Create Clip" button
   - Watch the progress bar
   - Download when complete

### Detailed Workflow

#### Step 1: Upload Video
```
┌─────────────────────────┐
│  📹 Drag & Drop Video   │
│      or Click to        │
│        Upload           │
└─────────────────────────┘
```

#### Step 2: Preview & Set Times
```
┌─────────────────────────┐
│  ▶️ Video Preview       │
│  ├─────────────────┤    │
│  Start: 0:30            │
│  End: 1:45              │
└─────────────────────────┘
```

#### Step 3: Process
```
┌─────────────────────────┐
│  Processing...          │
│  ████████████░░░ 75%    │
└─────────────────────────┘
```

#### Step 4: Download
```
┌─────────────────────────┐
│  ✅ Clip Ready!         │
│  Size: 2.3 MB           │
│  [💾 Download]          │
└─────────────────────────┘
```

## Technical Architecture

### Components

1. **Rust WASM Module** (`video_clip_rs.wasm`)
   - Time parsing (MM:SS, HH:MM:SS, seconds)
   - Output filename generation
   - Command generation

2. **FFmpeg.wasm**
   - Actual video processing
   - Codec operations
   - File I/O in browser

3. **JavaScript Glue**
   - File handling
   - UI interactions
   - Progress tracking
   - Blob management

### Data Flow

```
User Upload → FileReader API → Blob
                ↓
        Video Preview (HTML5)
                ↓
        Time Selection (Rust WASM)
                ↓
        FFmpeg.wasm Processing
                ↓
        Blob Output → Download
```

## Browser Requirements

### Minimum Requirements
- Chrome 90+ / Firefox 89+ / Safari 15+ / Edge 90+
- WebAssembly support
- SharedArrayBuffer support (for FFmpeg.wasm)
- 4GB RAM recommended

### Optimal Performance
- Chrome or Edge (Chromium-based)
- 8GB+ RAM
- Hardware acceleration enabled

## File Size Limitations

- **Recommended:** Videos under 500MB
- **Maximum:** ~2GB (browser-dependent)
- **Best Performance:** 10-100MB videos

For larger files, consider using the CLI version.

## Troubleshooting

### "Failed to initialize" Error
- Ensure you're using a modern browser
- Check that JavaScript is enabled
- Try refreshing the page

### Processing Takes Too Long
- Large videos may take several minutes
- Try shorter clips first
- Ensure sufficient RAM available

### Download Doesn't Start
- Check browser download settings
- Try a different browser
- Ensure sufficient disk space

### CORS Errors
- Must run from a web server (not file://)
- Use the provided Python server: `python3 -m http.server 8000`

## Advanced Usage

### Custom Time Formats

The tool accepts various time formats:
- `90` → 90 seconds
- `1:30` → 1 minute 30 seconds  
- `01:30:45` → 1 hour 30 minutes 45 seconds
- `2:45.5` → 2 minutes 45.5 seconds

### Keyboard Shortcuts

While video is focused:
- `Space` - Play/Pause
- `←/→` - Seek backward/forward
- `↑/↓` - Volume up/down

### Batch Processing

For multiple clips from the same video:
1. Upload once
2. Create first clip
3. Change times
4. Create next clip
5. Repeat without re-uploading

## Performance Tips

1. **Close other tabs** - Video processing is memory-intensive
2. **Use shorter clips** - Process in segments if needed
3. **Optimal formats** - MP4/H.264 processes fastest
4. **Disable extensions** - Ad blockers may interfere

## Security & Privacy

### What happens to your video?

1. **Upload**: File is read into browser memory
2. **Processing**: FFmpeg.wasm processes in a sandboxed environment
3. **Download**: Result saved to your Downloads folder
4. **Cleanup**: Memory freed when you close/refresh the page

### No data is:
- ❌ Sent to servers
- ❌ Stored permanently
- ❌ Accessible to website owner
- ❌ Tracked or analyzed

### All processing is:
- ✅ Local to your device
- ✅ Temporary (RAM only)
- ✅ Sandboxed in browser
- ✅ Cleared on page close

## Comparison with CLI Version

| Feature | Browser Version | CLI Version |
|---------|----------------|-------------|
| Installation | None needed | Rust required |
| File size limit | ~2GB | Unlimited |
| Processing speed | Good | Excellent |
| Privacy | Complete | Complete |
| Platform support | Any browser | Linux/Mac/Windows |
| FFmpeg features | Limited | Full |
| Batch processing | Manual | Scriptable |

## Development

### Building from Source

```bash
# Clone repository
git clone <repo-url>
cd video-clip-rs

# Install dependencies
./setup.sh

# Build WASM module
wasm-pack build --target web --features wasm --no-default-features

# Start server
python3 -m http.server 8000
```

### Customization

Edit `index_advanced.html` to:
- Change UI colors/layout
- Add new features
- Modify time formats
- Adjust file size limits

## Support

For issues or questions:
1. Check the troubleshooting section
2. Try the CLI version for comparison
3. Report issues on GitHub

## License

MIT - Free for personal and commercial use