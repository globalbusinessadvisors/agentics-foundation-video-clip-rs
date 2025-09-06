#!/bin/bash

echo "🎬 Video Clipper Setup Script"
echo "============================"
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Add WASM target
echo "🎯 Adding WebAssembly target..."
rustup target add wasm32-unknown-unknown

# Install wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    echo "📦 Installing wasm-pack..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build WASM module
echo "🔨 Building WebAssembly module..."
wasm-pack build --target web --features wasm --no-default-features

# Build CLI
echo "🔨 Building CLI tool..."
cargo build --release --features cli

# Install npm dependencies for FFmpeg.wasm (optional)
if command -v npm &> /dev/null; then
    echo "📦 Installing FFmpeg.wasm dependencies..."
    npm install
fi

echo ""
echo "✅ Setup complete!"
echo ""
echo "To use the web interface:"
echo "  1. Start a local server: python3 -m http.server 8000"
echo "  2. Open: http://localhost:8000/index_advanced.html"
echo ""
echo "To use the CLI:"
echo "  ./target/release/video-clip --help"
echo ""