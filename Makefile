# Makefile for regmsg project
# Provides convenient targets for building, testing, and cross-compiling

.PHONY: help build build-release build-debug clean test cross-compile-aarch64 docker-build docker-build-aarch64 docker-run docker-run-aarch64

# Default target
help:
	@echo "regmsg Makefile - Available targets:"
	@echo "  help                   - Show this help message"
	@echo "  build                  - Build in debug mode"
	@echo "  build-release          - Build in release mode (default)"
	@echo "  build-debug            - Build in debug mode"
	@echo "  clean                  - Clean build artifacts"
	@echo "  test                   - Run tests"
	@echo "  cross-compile-aarch64  - Cross-compile for aarch64 using traditional method"
	@echo "  docker-build           - Build using Docker (native architecture)"
	@echo "  docker-build-aarch64   - Build using Docker for aarch64"
	@echo "  docker-run             - Run regmsg in Docker (native architecture)"
	@echo "  docker-run-aarch64     - Run regmsg in Docker for aarch64"

# Build in release mode by default
build: build-release

# Build in release mode
build-release:
	cargo build --release

# Build in debug mode
build-debug:
	cargo build

# Clean build artifacts
clean:
	cargo clean

# Run tests
test:
	cargo test

# Cross-compile for aarch64 using traditional method
cross-compile-aarch64:
	@echo "Cross-compiling for aarch64..."
	@echo "Make sure you have the required target and tools installed:"
	@echo "  rustup target add aarch64-unknown-linux-gnu"
	@echo "  sudo apt install gcc-aarch64-linux-gnu pkg-config-aarch64-linux-gnu"
	@echo ""
	cargo build --target aarch64-unknown-linux-gnu --release

# Docker build for native architecture
docker-build:
	@echo "Building regmsg using Docker (native architecture)..."
	docker build --platform linux/amd64 -t regmsg-native -f Dockerfile.x86_64 .

# Docker build for aarch64
docker-build-aarch64:
	@echo "Building regmsg using Docker for aarch64..."
	docker build --platform linux/aarch64 -t regmsg-aarch64 -f Dockerfile .
	@echo ""
	@echo "Build completed! You can run the image with:"
	@echo "  docker run --rm -it regmsg-aarch64"

# Docker run native
docker-run:
	@echo "Running regmsg in Docker (native architecture)..."
	docker run --rm -it regmsg-native

# Docker run aarch64
docker-run-aarch64:
	@echo "Running regmsg in Docker for aarch64..."
	docker run --rm -it --platform linux/aarch64 regmsg-aarch64

# Convenience alias for Docker cross-compilation
docker-cross-compile: docker-build-aarch64
	@echo "Image built. To extract binaries, run:"
	@echo "  docker create --name temp_container regmsg-aarch64"
	@echo "  docker cp temp_container:/usr/local/bin/regmsg ./regmsg-aarch64"
	@echo "  docker cp temp_container:/usr/local/bin/regmsgd ./regmsgd-aarch64"
	@echo "  docker rm temp_container > /dev/null"