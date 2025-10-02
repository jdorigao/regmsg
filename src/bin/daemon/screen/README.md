# Display Manager for KMS/DRM and Wayland

This Rust project provides a library and utilities to manage display settings on Linux systems, supporting both KMS/DRM and Wayland backends. It allows users to list available display modes and outputs, retrieve current display settings (resolution, refresh rate, etc.), and set specific display configurations such as resolution, refresh rate, rotation, and more.

## Features

- **Backend Detection**: Automatically detects whether the system uses Wayland or KMS/DRM.
- **Display Mode Management**: List, retrieve, and set display modes (e.g., `1920x1080@60Hz`).
- **Output Management**: List and set active outputs (e.g., HDMI, DisplayPort).
- **Rotation Control**: Rotate the display to 0°, 90°, 180°, or 270°.
- **Screenshot Capture**: Capture screenshots (Wayland via `grim`, KMS/DRM partially implemented).
- **Touchscreen Mapping**: Map touchscreen input to the correct display (Wayland only).
- **Maximum Resolution**: Set displays to their maximum supported resolution within specified limits.

## Supported Backends

- **KMS/DRM**: Uses Direct Rendering Manager for low-level display control.
- **Wayland**: Integrates with the Wayland compositor (e.g., Sway) via `swayipc`.
