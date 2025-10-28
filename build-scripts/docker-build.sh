#!/bin/bash

# Docker-based cross-compilation script for regmsg
# This script uses Docker to build regmsg for different architectures without
# requiring local cross-compilation toolchains

set -e  # Exit immediately if a command exits with a non-zero status

# Default target
TARGET="aarch64"
DOCKERFILE="Dockerfile"
RELEASE_FLAG="--release"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --target)
            TARGET="$2"
            shift 2
            ;;
        --debug)
            RELEASE_FLAG=""
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --target TARGET    Target architecture (aarch64|x86_64) (default: aarch64)"
            echo "  --debug           Build in debug mode instead of release"
            echo "  --help            Show this help message"
            echo ""
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo "Building regmsg for target: $TARGET using Docker"

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo "Error: Docker is not installed or not in PATH"
    exit 1
fi

# Check if Docker daemon is running
if ! docker info &> /dev/null; then
    echo "Error: Docker daemon is not running. Please start Docker and try again."
    exit 1
fi

# Enable Docker BuildKit for better multi-platform support
export DOCKER_BUILDKIT=1

# Check Docker version to ensure it supports multi-platform builds
DOCKER_VERSION=$(docker version --format '{{.Server.Version}}' 2>/dev/null || echo "unknown")
echo "Docker version: $DOCKER_VERSION"

# Verify that the required Dockerfile exists
if [ "$TARGET" = "aarch64" ]; then
    DOCKERFILE="Dockerfile"
    BUILD_ARGS="--platform linux/aarch64"
    OUTPUT_DIR="target/aarch64-unknown-linux-gnu"
    echo "Using Dockerfile for aarch64 cross-compilation"
elif [ "$TARGET" = "x86_64" ]; then
    DOCKERFILE="Dockerfile.x86_64"
    BUILD_ARGS="--platform linux/amd64"
    OUTPUT_DIR="target/x86_64-unknown-linux-gnu"
    echo "Using Dockerfile for x86_64 native compilation"
else
    echo "Error: Unsupported target '$TARGET'. Supported targets: aarch64, x86_64"
    exit 1
fi

if [ ! -f "$DOCKERFILE" ]; then
    echo "Error: Dockerfile '$DOCKERFILE' not found"
    exit 1
fi

# Build the Docker image
echo "Building Docker image for $TARGET..."
docker build $BUILD_ARGS -t regmsg-builder-$TARGET -f $DOCKERFILE .

# Run the build inside the container
echo "Building regmsg inside Docker container..."
if [ "$TARGET" = "aarch64" ]; then
    docker run --rm \
        -v "$(pwd)":/usr/src/regmsg \
        -w /usr/src/regmsg \
        regmsg-builder-$TARGET \
        cargo build --target aarch64-unknown-linux-gnu $RELEASE_FLAG
else
    docker run --rm \
        -v "$(pwd)":/usr/src/regmsg \
        -w /usr/src/regmsg \
        regmsg-builder-$TARGET \
        cargo build $RELEASE_FLAG
fi

# Determine the output directory based on release/debug and target
if [ "$RELEASE_FLAG" = "--release" ]; then
    OUTPUT_DIR="${OUTPUT_DIR}/release"
else
    OUTPUT_DIR="${OUTPUT_DIR}/debug"
fi

echo ""
echo "Build completed successfully!"
echo "Binaries are located in: $OUTPUT_DIR/"
echo "Available binaries:"
if [ -d "$OUTPUT_DIR" ]; then
    ls -la "$OUTPUT_DIR/" | grep -E "(regmsg|regmsgd)" || echo "No binaries found in $OUTPUT_DIR/"
else
    echo "Output directory $OUTPUT_DIR does not exist"
fi

echo ""
echo "To copy binaries to your host system, you can run:"
echo "  docker create --name temp_container regmsg-builder-$TARGET"
echo "  docker cp temp_container:/usr/local/bin/regmsg ."
echo "  docker cp temp_container:/usr/local/bin/regmsgd ."
echo "  docker rm temp_container > /dev/null"