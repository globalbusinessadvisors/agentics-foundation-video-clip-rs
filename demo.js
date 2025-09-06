#!/usr/bin/env node

// Node.js demo script for testing WASM module
// Run with: node demo.js

import fs from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Read WASM file
const wasmPath = join(__dirname, 'pkg', 'video_clip_rs_bg.wasm');
const wasmBuffer = fs.readFileSync(wasmPath);

// Import and initialize WASM module
async function runDemo() {
    try {
        // Dynamic import of the JS bindings
        const wasmModule = await import('./pkg/video_clip_rs.js');
        
        // Initialize WASM with the buffer
        await wasmModule.default(wasmBuffer);
        
        console.log('🎬 Video Clipper WASM Demo');
        console.log('=' .repeat(50));
        
        // Test time parsing
        console.log('\n📊 Time Parsing Tests:');
        const times = ['1:30', '90', '2:45:30', '165'];
        for (const time of times) {
            const seconds = wasmModule.parse_time_to_seconds(time);
            const formatted = wasmModule.format_time_readable(seconds);
            console.log(`  ${time.padEnd(10)} → ${seconds}s → ${formatted}`);
        }
        
        // Test time range validation
        console.log('\n✅ Time Range Validation:');
        try {
            const duration = wasmModule.validate_time_range(60, 120);
            console.log(`  60s to 120s: Valid (duration: ${duration}s)`);
        } catch (e) {
            console.log(`  60s to 120s: Invalid - ${e}`);
        }
        
        try {
            const duration = wasmModule.validate_time_range(120, 60);
            console.log(`  120s to 60s: Valid (duration: ${duration}s)`);
        } catch (e) {
            console.log(`  120s to 60s: Invalid - ${e}`);
        }
        
        // Generate FFmpeg command
        console.log('\n🎞️ FFmpeg Command Generation:');
        const command = wasmModule.generate_ffmpeg_command(
            'input.mp4',
            'output.mp4',
            '1:30',
            '2:45'
        );
        console.log(`  ${command}`);
        
        // Test WasmVideoClipper class
        console.log('\n🔧 WasmVideoClipper Class:');
        const clipper = new wasmModule.WasmVideoClipper();
        
        const outputFile = clipper.generate_output_filename(
            'test_video.mp4',
            '1:30',
            '2:45'
        );
        console.log(`  Generated output filename: ${outputFile}`);
        
        // Prepare clip request
        const clipRequest = {
            input_file: 'sample_video.mp4',
            start_time: '0:30',
            end_time: '1:45'
        };
        
        const clipResult = wasmModule.prepare_clip(clipRequest);
        console.log('\n📋 Clip Result:');
        console.log(`  Input: ${clipResult.input_file}`);
        console.log(`  Output: ${clipResult.output_file}`);
        console.log(`  Duration: ${clipResult.duration}s`);
        console.log(`  Command: ${clipResult.command}`);
        
        console.log('\n✨ Demo completed successfully!');
        
    } catch (error) {
        console.error('❌ Error:', error);
    }
}

runDemo();