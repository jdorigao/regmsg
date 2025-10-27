# regmsg

**regmsg** is a command-line utility and daemon for managing screen resolution and display settings on Linux systems, with support for both Wayland compositors and KMS/DRM backends.

## Features

- **Display Management**: Manage screen resolution, orientation, refresh rate, and output settings
- **Dual Backend Support**: Works with both KMS/DRM and Wayland (Sway) backends
- **Daemon Architecture**: Client-server architecture with persistent daemon for efficient display management
- **Advanced Tracing**: Comprehensive logging with dual output to file and console, supporting environment-based filtering
- **Lightweight and Optimized**: Designed for performance with async I/O
- **Scripting Support**: Designed for automation and integration in tiling window manager environments

## Architecture

The system consists of two main components:
- **regmsg**: Command-line client that communicates with the daemon
- **regmsgd**: Background daemon that manages display settings using ZeroMQ for IPC

## Usage

### Client (regmsg)
```bash
regmsg [OPTIONS] [COMMAND]
``` 

### Daemon (regmsgd)
```bash
regmsgd
```

For a full list of options and commands, use:
```bash
regmsg --help
```

Common commands include:
- `listModes`: List available display modes
- `currentMode`: Get current display mode
- `setMode <mode>`: Set display mode (e.g., "1920x1080@60")
- `setRotation <rotation>`: Set display rotation (0, 90, 180, 270)
- `getScreenshot`: Take a screenshot
- `mapTouchScreen`: Map touchscreen to display

## Building

To build **regmsg**, you need to have the Rust toolchain installed. You can install it using [rustup](https://rustup.rs/).

```bash
git clone https://github.com/REG-Linux/regmsg.git
cd regmsg
cargo build --release
```

The compiled binaries will be located in the `target/release` directory.

## Logging and Tracing

The daemon implements a robust tracing mechanism with:

- **Dual Output**: Logs are written both to console/stdout for debugging and to a file (`/var/log/regmsg.log` by default) for persistent record keeping
- **Environment Filtering**: Supports `RUST_LOG` environment variable for controlling log levels (e.g., `RUST_LOG=debug`)
- **Non-blocking I/O**: Uses non-blocking writers to prevent logging from affecting system performance
- **No Automatic Rotation**: Single log file approach to allow system-level log management tools like `logrotate`
- **ANSI Support**: Colored output to console and plain text to file for optimal readability

## Requirements

- Rust (edition 2024 or later)
- A Wayland compositor (e.g., Sway) or DRM support for KMS/DRM backend
- ZeroMQ library for IPC communication

## License

This project is licensed under the [MIT License](LICENSE).
