#!/bin/bash

# Weather Station Docker Run Script for Raspberry Pi
# This script helps run the weather station container with proper configuration

set -e

# Configuration
IMAGE_NAME="weather-station"
TAG="latest"
CONTAINER_NAME="weather-station"

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

# Show usage
usage() {
    echo "Usage: $0 [OPTIONS] [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  start       Start the weather station container"
    echo "  stop        Stop the weather station container"
    echo "  restart     Restart the weather station container"
    echo "  logs        Show container logs"
    echo "  status      Show container status"
    echo "  shell       Open shell in running container"
    echo "  remove      Remove the container"
    echo ""
    echo "Options:"
    echo "  -d, --detach        Run in detached mode (background)"
    echo "  -f, --follow        Follow logs (tail -f)"
    echo "  -h, --help          Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 start            # Start container in foreground"
    echo "  $0 start -d         # Start container in background"
    echo "  $0 logs -f          # Follow logs"
    echo "  $0 status           # Check if container is running"
}

# Check Docker installation
check_docker() {
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed or not in PATH"
        exit 1
    fi
}

# Check if container exists
container_exists() {
    docker ps -a --format "table {{.Names}}" | grep -q "^${CONTAINER_NAME}$"
}

# Check if container is running
container_running() {
    docker ps --format "table {{.Names}}" | grep -q "^${CONTAINER_NAME}$"
}

# Check USB device
check_usb_device() {
    if lsusb | grep -q "1941:8021"; then
        print_success "Weather station USB device detected"
        return 0
    else
        print_warning "Weather station USB device not detected"
        print_warning "Make sure the device is connected and you have proper permissions"
        return 1
    fi
}

# Start container
start_container() {
    local detach_mode=false
    
    # Parse start options
    while [[ $# -gt 0 ]]; do
        case $1 in
            -d|--detach)
                detach_mode=true
                shift
                ;;
            *)
                print_error "Unknown option for start: $1"
                exit 1
                ;;
        esac
    done
    
    # Check if container is already running
    if container_running; then
        print_warning "Container is already running"
        print_status "Use '$0 restart' to restart or '$0 stop' to stop"
        return 0
    fi
    
    # Remove existing stopped container
    if container_exists; then
        print_status "Removing existing stopped container..."
        docker rm $CONTAINER_NAME > /dev/null
    fi
    
    # Check USB device
    check_usb_device
    
    # Check config file
    if [ ! -f "config.toml" ]; then
        print_error "config.toml not found in current directory"
        print_status "Make sure you're in the weather-station directory"
        exit 1
    fi
    
    # Build run command
    RUN_CMD="docker run"
    
    if [ "$detach_mode" = true ]; then
        RUN_CMD="$RUN_CMD -d"
    else
        RUN_CMD="$RUN_CMD -it"
    fi
    
    RUN_CMD="$RUN_CMD --name $CONTAINER_NAME"
    RUN_CMD="$RUN_CMD --restart unless-stopped"
    RUN_CMD="$RUN_CMD --privileged"
    RUN_CMD="$RUN_CMD -v \$(pwd)/config.toml:/app/config.toml:ro"
    RUN_CMD="$RUN_CMD -v /dev/bus/usb:/dev/bus/usb"
    RUN_CMD="$RUN_CMD --network host"
    RUN_CMD="$RUN_CMD -e RUST_LOG=info"
    RUN_CMD="$RUN_CMD -e TZ=Europe/Moscow"
    RUN_CMD="$RUN_CMD ${IMAGE_NAME}:${TAG}"
    
    print_status "Starting weather station container..."
    print_status "Command: $RUN_CMD"
    
    if eval $RUN_CMD; then
        if [ "$detach_mode" = true ]; then
            print_success "Container started in background"
            print_status "Use '$0 logs' to view logs"
        else
            print_success "Container started"
        fi
    else
        print_error "Failed to start container"
        exit 1
    fi
}

# Stop container
stop_container() {
    if container_running; then
        print_status "Stopping weather station container..."
        docker stop $CONTAINER_NAME > /dev/null
        print_success "Container stopped"
    else
        print_warning "Container is not running"
    fi
}

# Show logs
show_logs() {
    local follow_mode=false
    
    # Parse logs options
    while [[ $# -gt 0 ]]; do
        case $1 in
            -f|--follow)
                follow_mode=true
                shift
                ;;
            *)
                print_error "Unknown option for logs: $1"
                exit 1
                ;;
        esac
    done
    
    if ! container_exists; then
        print_error "Container does not exist"
        exit 1
    fi
    
    if [ "$follow_mode" = true ]; then
        print_status "Following logs (Ctrl+C to exit)..."
        docker logs -f $CONTAINER_NAME
    else
        docker logs $CONTAINER_NAME
    fi
}

# Show status
show_status() {
    if container_running; then
        print_success "Container is running"
        docker ps --filter "name=$CONTAINER_NAME" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
        echo ""
        print_status "Recent logs:"
        docker logs --tail 10 $CONTAINER_NAME
    elif container_exists; then
        print_warning "Container exists but is not running"
        docker ps -a --filter "name=$CONTAINER_NAME" --format "table {{.Names}}\t{{.Status}}"
    else
        print_warning "Container does not exist"
    fi
}

# Open shell
open_shell() {
    if container_running; then
        print_status "Opening shell in running container..."
        docker exec -it $CONTAINER_NAME /bin/bash
    else
        print_error "Container is not running"
        print_status "Start the container first with '$0 start'"
        exit 1
    fi
}

# Remove container
remove_container() {
    if container_running; then
        print_status "Stopping running container..."
        docker stop $CONTAINER_NAME > /dev/null
    fi
    
    if container_exists; then
        print_status "Removing container..."
        docker rm $CONTAINER_NAME > /dev/null
        print_success "Container removed"
    else
        print_warning "Container does not exist"
    fi
}

# Main script
check_docker

# Change to project root
cd "$(dirname "$0")/.."

# Parse command line arguments
COMMAND=""
FOLLOW=false
DETACH=false

while [[ $# -gt 0 ]]; do
    case $1 in
        start|stop|restart|logs|status|shell|remove)
            if [ -n "$COMMAND" ]; then
                print_error "Multiple commands specified"
                exit 1
            fi
            COMMAND="$1"
            shift
            ;;
        -f|--follow)
            FOLLOW=true
            shift
            ;;
        -d|--detach)
            DETACH=true
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

# Default command
if [ -z "$COMMAND" ]; then
    COMMAND="start"
fi

# Execute command
case $COMMAND in
    start)
        if [ "$DETACH" = true ]; then
            start_container -d
        else
            start_container
        fi
        ;;
    stop)
        stop_container
        ;;
    restart)
        stop_container
        sleep 2
        if [ "$DETACH" = true ]; then
            start_container -d
        else
            start_container
        fi
        ;;
    logs)
        if [ "$FOLLOW" = true ]; then
            show_logs -f
        else
            show_logs
        fi
        ;;
    status)
        show_status
        ;;
    shell)
        open_shell
        ;;
    remove)
        remove_container
        ;;
    *)
        print_error "Unknown command: $COMMAND"
        usage
        exit 1
        ;;
esac
