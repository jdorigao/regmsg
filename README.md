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

## License

This project is licensed under the [MIT License](LICENSE).
