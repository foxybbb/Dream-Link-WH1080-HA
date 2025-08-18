#!/bin/bash

# Weather Station Docker Build Script for Raspberry Pi
# This script builds the Docker image for the target Raspberry Pi architecture

set -e

# Configuration
IMAGE_NAME="weather-station"
TAG="latest"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Detect architecture
detect_arch() {
    local arch=$(uname -m)
    case $arch in
        x86_64)
            echo "linux/amd64"
            ;;
        aarch64|arm64)
            echo "linux/arm64"
            ;;
        armv7l|armhf)
            echo "linux/arm/v7"
            ;;
        *)
            print_error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac
}

# Show usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -a, --arch ARCH     Target architecture (auto-detected if not specified)"
    echo "                      Supported: linux/arm64, linux/arm/v7, linux/amd64"
    echo "  -t, --tag TAG       Image tag (default: latest)"
    echo "  -m, --multi         Build multi-architecture image"
    echo "  -p, --push          Push to registry after build"
    echo "  -h, --help          Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                           # Build for current architecture"
    echo "  $0 -a linux/arm64           # Build for Raspberry Pi 4/5"
    echo "  $0 -a linux/arm/v7          # Build for Raspberry Pi 3/Zero"
    echo "  $0 -m -p                     # Build multi-arch and push"
}

# Parse command line arguments
ARCH=""
MULTI_ARCH=false
PUSH=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -a|--arch)
            ARCH="$2"
            shift 2
            ;;
        -t|--tag)
            TAG="$2"
            shift 2
            ;;
        -m|--multi)
            MULTI_ARCH=true
            shift
            ;;
        -p|--push)
            PUSH=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Auto-detect architecture if not specified
if [ -z "$ARCH" ] && [ "$MULTI_ARCH" = false ]; then
    ARCH=$(detect_arch)
    print_status "Auto-detected architecture: $ARCH"
fi

# Change to project root
cd "$(dirname "$0")/.."

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed or not in PATH"
    exit 1
fi

# Check if buildx is available for multi-arch builds
if [ "$MULTI_ARCH" = true ]; then
    if ! docker buildx version &> /dev/null; then
        print_error "Docker Buildx is required for multi-architecture builds"
        exit 1
    fi
fi

print_status "Building Weather Station Docker image..."
print_status "Image: ${IMAGE_NAME}:${TAG}"

if [ "$MULTI_ARCH" = true ]; then
    print_status "Building multi-architecture image..."
    
    # Create buildx builder if it doesn't exist
    if ! docker buildx ls | grep -q weather-builder; then
        print_status "Creating buildx builder..."
        docker buildx create --name weather-builder --use
    else
        docker buildx use weather-builder
    fi
    
    # Build command for multi-arch
    BUILD_CMD="docker buildx build"
    BUILD_CMD="$BUILD_CMD --platform linux/arm64,linux/arm/v7"
    BUILD_CMD="$BUILD_CMD -t ${IMAGE_NAME}:${TAG}"
    
    if [ "$PUSH" = true ]; then
        BUILD_CMD="$BUILD_CMD --push"
    else
        BUILD_CMD="$BUILD_CMD --load"
    fi
    
    BUILD_CMD="$BUILD_CMD ."
else
    # Build command for single arch
    BUILD_CMD="docker build"
    BUILD_CMD="$BUILD_CMD --platform ${ARCH}"
    BUILD_CMD="$BUILD_CMD -t ${IMAGE_NAME}:${TAG}"
    BUILD_CMD="$BUILD_CMD ."
fi

print_status "Executing: $BUILD_CMD"

# Execute build
if eval $BUILD_CMD; then
    print_success "Docker image built successfully!"
    
    # Show image info
    if [ "$MULTI_ARCH" = false ]; then
        docker images ${IMAGE_NAME}:${TAG}
    fi
    
    # Push if requested and not multi-arch (multi-arch pushes during build)
    if [ "$PUSH" = true ] && [ "$MULTI_ARCH" = false ]; then
        print_status "Pushing image to registry..."
        docker push ${IMAGE_NAME}:${TAG}
        print_success "Image pushed successfully!"
    fi
    
    print_success "Build completed!"
    echo ""
    print_status "To run the container:"
    echo "  docker-compose up -d"
    echo ""
    print_status "To run manually:"
    echo "  docker run -d --name weather-station \\"
    echo "    --privileged \\"
    echo "    -v \$(pwd)/config.toml:/app/config.toml:ro \\"
    echo "    -v /dev/bus/usb:/dev/bus/usb \\"
    echo "    --network host \\"
    echo "    ${IMAGE_NAME}:${TAG}"
    
else
    print_error "Build failed!"
    exit 1
fi
