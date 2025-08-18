#!/bin/bash

# Docker Build Fix Script for Raspberry Pi Weather Station
# This script ensures all necessary files are present and builds the container correctly

set -e

echo "🔧 Docker Build Fix for Weather Station"
echo "======================================"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

echo "📋 Checking required files..."

# Check for required files
REQUIRED_FILES=(
    "Cargo.toml"
    "Cargo.lock" 
    "config.toml"
    "Dockerfile"
    "docker/41-weather-device.rules"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        echo "✅ $file - Found"
    else
        echo "❌ $file - Missing!"
        exit 1
    fi
done

echo "📁 Checking src directory..."
if [ -d "src" ] && [ -f "src/main.rs" ]; then
    echo "✅ src/main.rs - Found"
else
    echo "❌ src/main.rs - Missing!"
    exit 1
fi

echo ""
echo "🧹 Cleaning previous builds..."
# Remove any existing containers and images
docker compose down 2>/dev/null || true
docker rmi weather-station:latest 2>/dev/null || echo "No existing image to remove"

echo ""
echo "🐳 Building Docker image..."
echo "Platform: $(uname -m)"
echo "Architecture: $(arch)"

# Build for current platform (native compilation on Pi)
echo "Building natively on Raspberry Pi..."
docker compose build --no-cache

echo ""
echo "🚀 Starting container..."
docker compose up -d

echo ""
echo "✅ Build completed successfully!"
echo ""
echo "📊 Check status:"
echo "  docker compose ps"
echo "  docker compose logs -f"
echo ""
echo "🛠️ Useful commands:"
echo "  docker compose down    # Stop container"
echo "  docker compose restart # Restart container"
echo "  docker compose logs    # View logs"
