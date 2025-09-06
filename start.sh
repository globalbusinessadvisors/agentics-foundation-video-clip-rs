#!/bin/bash

# Video Clipper Pro - Startup Script
# This script handles port conflicts and starts the server cleanly

echo "🎬 Starting Video Clipper Pro..."

# Kill any existing HTTP servers on port 8080
echo "🔄 Cleaning up existing servers..."
pkill -f "python.*http.server.*8080" 2>/dev/null || true
lsof -ti:8080 | xargs kill -9 2>/dev/null || true

# Wait a moment for cleanup
sleep 1

# Verify WASM package exists
if [ ! -f "pkg/video_clip_rs_bg.wasm" ]; then
    echo "🔧 WASM package not found, building..."
    if command -v wasm-pack >/dev/null 2>&1; then
        wasm-pack build --target web --out-dir pkg --features wasm --no-default-features
    else
        echo "❌ wasm-pack not found. Please install it first:"
        echo "curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
        exit 1
    fi
fi

# Ensure index.html exists for default serving
if [ ! -f "index.html" ] && [ -f "index_unified.html" ]; then
    echo "📄 Creating index.html from index_unified.html..."
    cp index_unified.html index.html
fi

# Start the server
echo "🚀 Starting server on http://localhost:8080..."
echo "📂 Serving: $(pwd)"
echo "🌐 Open: http://localhost:8080/"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

# Start Python HTTP server
python3 -m http.server 8080