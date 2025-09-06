# 🎬 Video Clipper Pro - Usage Guide

## How to Create Video Clips

### Step 1: Start the Application
```bash
./start.sh
```
The browser will automatically open to: `http://localhost:8080/index_unified.html`

### Step 2: Upload Your Video
1. **Click the green upload area** or **drag & drop** your video file
2. **Supported formats**: MP4, MOV, AVI, MKV, WebM, and most video formats
3. **File size**: Works best with files under 2GB

### Step 3: Set Your Clip Times

**Multiple time formats supported:**
- **Minutes:Seconds** - `1:30` (1 minute 30 seconds)
- **Hours:Minutes:Seconds** - `2:15:45` (2 hours 15 minutes 45 seconds)
- **Seconds only** - `90` (90 seconds)
- **With units** - `30s`, `5m`, `2h`, `1h30m45s`
- **Flexible** - `1h 30m 45s` (with spaces)

**Quick Actions Available:**
- **First 5s** - Click to set 0:00 to 0:05
- **First 10s** - Click to set 0:00 to 0:10  
- **First 30s** - Click to set 0:00 to 0:30
- **First 60s** - Click to set 0:00 to 1:00
- **Use Current Time** - Set start/end to current video position

### Step 4: Create and Download

1. **Click "✂️ Create Clip"** - The app will process your video
2. **Watch the progress** - Real-time status and progress bar
3. **Click "⬇️ Download Clip"** - MP4 file saves to your Downloads folder

## Example Usage

### Creating a 30-second highlight clip:
1. Upload your video
2. Enter start time: `2:15` (2 minutes 15 seconds)
3. Enter end time: `2:45` (2 minutes 45 seconds)  
4. Click "Create Clip"
5. Download your `video_clip.mp4`

### Creating a longer segment:
1. Upload your video
2. Enter start time: `1h30m` (1 hour 30 minutes)
3. Enter end time: `1h45m30s` (1 hour 45 minutes 30 seconds)
4. Click "Create Clip"
5. Download your `video_clip.mp4`

## Features

### ⚡ **Real-time Validation**
- Instant feedback on time format errors
- WASM-powered parsing for speed
- Helpful error messages with format suggestions

### 🎯 **Precise Timing** 
- Frame-accurate clipping
- Multiple time format support
- Visual preview with seek controls

### 🚀 **Fast Processing**
- Browser-based processing (no uploads needed)
- Real-time progress tracking
- Automatic MP4 output

### 🔒 **Privacy**
- 100% local processing
- Videos never leave your computer
- No server uploads required

## Troubleshooting

### **Video won't upload:**
- Check file format (MP4, MOV, AVI, MKV work best)
- Ensure file size is under 2GB
- Try a different browser (Chrome/Firefox recommended)

### **Invalid time format error:**
- Use formats like: `1:30`, `90s`, `2h15m`
- End time must be after start time
- Both fields are required

### **Processing fails:**
- Try shorter clips first
- Ensure stable internet (for FFmpeg.js loading)
- Refresh page and try again

### **Download doesn't work:**
- Check browser download permissions
- Ensure pop-ups are allowed
- Try right-click → "Save link as..."

## Technical Details

**Dual-Engine Architecture:**
- **Rust WASM**: Lightning-fast time parsing and validation
- **FFmpeg.js**: Browser-based video processing

**Supported Input Formats:**
- MP4, MOV, AVI, MKV, WebM, OGV, 3GP, FLV

**Output Format:**
- MP4 with H.264 video codec (compatible with all devices)

**Performance:**
- Small clips (under 1 minute): Process in 5-15 seconds
- Medium clips (1-5 minutes): Process in 15-60 seconds  
- Large clips (5+ minutes): May take several minutes

**Browser Compatibility:**
- ✅ Chrome 90+ (Recommended)
- ✅ Firefox 88+ (Good)
- ✅ Safari 14+ (Good)
- ✅ Edge 90+ (Good)

---

**Need help?** The interface provides real-time feedback and error messages to guide you through the process!