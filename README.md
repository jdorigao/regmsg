# regmsg

**regmsg** is a command-line utility for managing screen resolution and display settings on Wayland compositors, with a focus on DRM and Sway integration.

## Features

- Manage screen resolution and orientation
- Interact with Wayland compositors like Sway via IPC
- Lightweight and optimized for performance
- Designed for scripting and automation in tiling window manager environments

## Usage

```bash
regmsg [OPTIONS] [COMMAND]
``` 
For a full list of options and commands, use:

```bash
regmsg --help
```
## Building
To build **regmsg**, you need to have the Rust toolchain installed. You can install it using [rustup](https://rustup.rs/).
```bash 
git clone https://github.com/jdorigao/regmsg.git
cd regmsg
cargo build --release
```
The compiled binary will be located in the `target/release` directory.

## Requirements

- Rust (edition 2024 or later)
- A Wayland compositor (e.g., Sway)
- DRM support

## Cross-compilation

You have multiple options for cross-compilation to aarch64 (ARM64):

### Option 1: Direct cross-compilation (Traditional method)

1. Install the aarch64 target:
```bash
rustup target add aarch64-unknown-linux-gnu
```

2. Install cross-compilation tools:
```bash
# On Ubuntu/Debian:
sudo apt install gcc-aarch64-linux-gnu pkg-config-aarch64-linux-gnu

# On Fedora/RHEL:
sudo dnf install gcc-aarch64-linux-gnu pkg-config

# On Arch Linux:
sudo pacman -S gcc-aarch64-linux-gnu
```

3. Set up environment variables for cross-compilation (Linux):
```bash
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_PATH_aarch64_unknown_linux_gnu=/usr/lib/aarch64-linux-gnu/pkgconfig
export PKG_CONFIG_SYSROOT_DIR=/
```

4. Build for aarch64:
```bash
cargo build --target aarch64-unknown-linux-gnu --release
```

The cross-compiled binaries will be located in `target/aarch64-unknown-linux-gnu/release/`.

### Option 2: Docker-based cross-compilation (Recommended)

Using Docker makes cross-compilation easier as it handles all dependencies:

1. Make sure Docker is installed and running
2. Use the provided Docker build script:

```bash
./build-scripts/docker-build.sh --target aarch64
```

Or build manually with Docker:
```bash
docker build --platform linux/aarch64 -t regmsg-aarch64 -f Dockerfile .
```

### Option 3: Using Docker Compose

You can also use Docker Compose for building and running:

```bash
# Build for aarch64
docker-compose run regmsg-builder-aarch64

# Build for x86_64
docker-compose run regmsg-builder-native
```

### Using the build scripts

For convenience, use the provided build scripts:

```bash
# For traditional cross-compilation:
./build-scripts/cross-compile.sh --target aarch64-unknown-linux-gnu

# For Docker-based build:
./build-scripts/docker-build.sh --target aarch64
```

## License

This project is licensed under the [MIT License](LICENSE).
