#!/bin/bash

# Build script to bypass Cursor proxy issues
# This script creates a clean environment for Rust compilation

echo "🦀 Building Weather Station (Rust Port)"
echo "========================================="

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Cargo.toml not found. Make sure you're in the weather-station directory."
    exit 1
fi

# Function to try different approaches to run cargo
try_cargo() {
    echo "🔧 Attempting to build with cargo..."
    
    # Try 1: Use system package manager's rust installation
    if command -v /usr/bin/cargo >/dev/null 2>&1; then
        echo "📦 Trying system cargo..."
        /usr/bin/cargo build --release
        if [ $? -eq 0 ]; then
            echo "✅ Build successful with system cargo!"
            return 0
        fi
    fi
    
    # Try 2: Manual compilation with rustc
    echo "🛠️  System cargo failed, trying manual compilation..."
    
    # First, let's create a simple version check
    if ! command -v rustc >/dev/null 2>&1; then
        echo "❌ Error: rustc not found. Please install Rust using your package manager:"
        echo "   sudo pacman -S rust cargo"
        echo "   or"
        echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    echo "📋 Creating manual build..."
    
    # Create a temporary directory for dependencies
    mkdir -p target/manual-build
    
    echo "❌ Manual compilation with external dependencies is complex."
    echo "📝 Please install Rust via your system package manager instead:"
    echo ""
    echo "   sudo pacman -S rust cargo"
    echo ""
    echo "Then run: cargo build --release"
    
    return 1
}

# Try to build
try_cargo

if [ $? -eq 0 ]; then
    echo ""
    echo "🎉 Build completed successfully!"
    echo "📁 Binary location: ./target/release/weather-station"
    echo ""
    echo "🚀 To run the weather station:"
    echo "   ./target/release/weather-station"
    echo ""
    echo "⚠️  Note: Make sure your weather station is connected via USB"
    echo "   and you have proper USB permissions (see README.md)"
else
    echo ""
    echo "❌ Build failed. Please see the instructions above."
    echo "📖 Check README.md for detailed setup instructions."
fi 