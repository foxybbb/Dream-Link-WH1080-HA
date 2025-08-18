# Multi-stage Dockerfile for Raspberry Pi Weather Station
# Supports both ARM64 (Pi 4/5) and ARM32 (Pi 3/Zero) architectures

# Build stage
FROM --platform=$BUILDPLATFORM rust:1.75-slim-bullseye AS builder

# Install cross-compilation dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libusb-1.0-0-dev \
    libudev-dev \
    gcc-aarch64-linux-gnu \
    gcc-arm-linux-gnueabihf \
    && rm -rf /var/lib/apt/lists/*

# Set up cross-compilation targets
ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
    "linux/arm64") \
        rustup target add aarch64-unknown-linux-gnu && \
        echo "aarch64-unknown-linux-gnu" > /target_triple \
        ;; \
    "linux/arm/v7") \
        rustup target add armv7-unknown-linux-gnueabihf && \
        echo "armv7-unknown-linux-gnueabihf" > /target_triple \
        ;; \
    *) \
        echo "Unsupported platform: $TARGETPLATFORM" && exit 1 \
        ;; \
    esac

# Set up cross-compilation environment
RUN TARGET_TRIPLE=$(cat /target_triple) && \
    case "$TARGET_TRIPLE" in \
    "aarch64-unknown-linux-gnu") \
        echo '[target.aarch64-unknown-linux-gnu]' > ~/.cargo/config.toml && \
        echo 'linker = "aarch64-linux-gnu-gcc"' >> ~/.cargo/config.toml \
        ;; \
    "armv7-unknown-linux-gnueabihf") \
        echo '[target.armv7-unknown-linux-gnueabihf]' > ~/.cargo/config.toml && \
        echo 'linker = "arm-linux-gnueabihf-gcc"' >> ~/.cargo/config.toml \
        ;; \
    esac

# Create app directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN TARGET_TRIPLE=$(cat /target_triple) && \
    cargo build --release --target $TARGET_TRIPLE && \
    rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN TARGET_TRIPLE=$(cat /target_triple) && \
    cargo build --release --target $TARGET_TRIPLE && \
    cp target/$TARGET_TRIPLE/release/weather-station /weather-station

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libusb-1.0-0 \
    libudev1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd -r -s /bin/false weather && \
    mkdir -p /app/config && \
    chown -R weather:weather /app

# Copy the binary from builder stage
COPY --from=builder /weather-station /app/weather-station
RUN chmod +x /app/weather-station

# Copy default configuration
COPY config.toml /app/config/

# Create udev rules for USB access
RUN mkdir -p /etc/udev/rules.d/
COPY docker/41-weather-device.rules /etc/udev/rules.d/

# Set working directory
WORKDIR /app

# Create volume for configuration
VOLUME ["/app/config"]

# Switch to non-root user
USER weather

# Health check
HEALTHCHECK --interval=60s --timeout=10s --start-period=30s --retries=3 \
    CMD pgrep weather-station || exit 1

# Run the application
CMD ["./weather-station"]
