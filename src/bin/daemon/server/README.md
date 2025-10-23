# Server Module

This directory contains the core server components for the regmsg daemon, which manages display settings for REG Linux systems. The server uses a modular architecture to handle client requests and communicate with display backends.

## Architecture

The server module is organized into the following main components:

### 1. Command Registry (`command_registry.rs`)

The command registry manages dynamic command registration and execution. It provides:

- **Command Registration**: Register commands dynamically with name, description, and handler
- **Command Execution**: Execute registered commands with proper argument validation
- **Error Handling**: Comprehensive error handling with different error types
- **Macro Support**: Helper macros for creating different types of command handlers

#### Supported Commands

- `listModes`: List available display modes
- `listOutputs`: List available display outputs
- `currentMode`: Get current display mode
- `currentOutput`: Get current output
- `currentResolution`: Get current resolution
- `currentRotation`: Get current rotation
- `currentRefresh`: Get current refresh rate
- `currentBackend`: Get current display backend
- `setMode <mode>`: Set display mode (e.g., "1920x1080@60")
- `setOutput <output>`: Set display output
- `setRotation <rotation>`: Set display rotation (0, 90, 180, 270)
- `getScreenshot`: Take a screenshot
- `mapTouchScreen`: Map touchscreen to display
- `minTomaxResolution`: Set resolution to maximum

### 2. Command Handler (`commands.rs`)

The command handler module registers and manages all available commands:

- **Command Registration**: Initializes and registers all supported commands with the registry
- **Command Types**: Provides different command types for various use cases (simple, with arguments, screen-aware)
- **Validation**: Ensures commands receive correct arguments and parameters

### 3. Server (`server.rs`)

The main server implementation handles ZeroMQ communication:

- **ZeroMQ Integration**: Uses REP socket for request-reply pattern
- **Message Loop**: Processes incoming messages in an infinite loop
- **Socket Management**: Handles socket creation, binding, and cleanup
- **Client Communication**: Receives commands and sends formatted responses

## Communication Protocol

The server uses ZeroMQ IPC communication with the following characteristics:

- **Transport**: IPC (`ipc:///var/run/regmsgd.sock`)
- **Pattern**: Request-Reply (REQ-REP)
- **Message Format**: UTF-8 encoded strings
- **Socket Path**: `/var/run/regmsgd.sock`

## Usage

The server module is designed to work with the main daemon entry point and integrates with the screen management modules to provide display configuration capabilities. It supports both Wayland and KMS/DRM backends through the screen module abstraction.

## Error Handling

The module implements comprehensive error handling through the `CommandError` enum with the following error types:

- `InvalidArguments`: When commands receive invalid or missing arguments
- `UnknownCommand`: When an unrecognized command is received
- `ExecutionError`: When command execution fails
- `EmptyCommand`: When an empty command string is received

## Dependencies

### Rust Dependencies
- `zeromq`: For ZeroMQ communication
- `async-std`: For async runtime support
- Screen management modules: For actual display operations

### System Dependencies
- ZeroMQ library: Required for IPC communication
- DRM/KMS libraries: For direct display management (when using KMS backend)
- Wayland libraries: For Wayland compositor support
- X11 libraries: For X11 display server support (if applicable)

## Command Types

The server supports different types of commands with varying argument requirements:

### 1. Simple Command (no arguments)

Commands that don't require arguments and return a string.

```rust
registry.register(
    "currentBackend",
    simple_command!(
        "currentBackend",
        "Displays the current window system",
        || screen::current_backend()
    ),
);
```

**Usage example:**
```bash
regmsg currentBackend
# Output: Wayland
```

### 2. Screen Command (optional screen parameter)

Commands that can receive an optional screen name parameter.

```rust
registry.register(
    "currentMode",
    screen_command(
        "Displays the current display mode",
        |screen| screen::current_mode(screen),
    ),
);
```

**Usage example:**
```bash
regmsg currentMode
# Output: 1920x1080@60

regmsg currentMode HDMI-1
# Output: 2560x1440@144
```

### 3. Setter Command with Screen

Commands that modify settings with a required value and optional screen parameter.

```rust
registry.register(
    "setMode",
    screen_setter_command(
        "Sets the display mode",
        |screen, mode| screen::set_mode(screen, mode),
    ),
);
```

**Usage example:**
```bash
regmsg setMode 1920x1080@60
# Sets mode for current screen

regmsg setMode 1920x1080@60 HDMI-1
# Sets mode for HDMI-1
```

### 4. Fixed Arguments Command

Commands that require a specific number of arguments.

```rust
registry.register(
    "setOutput",
    Box::new(ArgCommand {
        name: "setOutput".to_string(),
        description: "Sets the output resolution".to_string(),
        expected_args: 1,
        executor: Box::new(|args| screen::set_output(args[0])),
    }),
);
```

**Usage example:**
```bash
regmsg setOutput HDMI-1
```

## Adding New Commands

### Step 1: Implement the Function in Screen Module

Add your function in `src/bin/daemon/screen/mod.rs`:

```rust
pub fn my_new_feature(screen: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    // Your implementation here
    Ok(format!("Feature executed for {:?}", screen))
}
```

### Step 2: Register the Command

Add the registration in `src/bin/daemon/server/commands.rs`:

```rust
pub fn init_commands() -> CommandRegistry {
    let mut registry = CommandRegistry::new();
    
    // ... existing commands ...
    
    // Your new command
    registry.register(
        "myNewFeature",
        screen_command(
            "Description of what this command does",
            |screen| screen::my_new_feature(screen),
        ),
    );
    
    registry
}
```

### Step 3: Use the Command

```bash
regmsg myNewFeature
regmsg myNewFeature HDMI-1
```

## Complete Examples

### Example 1: Simple Status Command

```rust
// In screen/mod.rs
pub fn gpu_temperature() -> Result<String, Box<dyn std::error::Error>> {
    // Read GPU temperature
    let temp = read_gpu_temp()?;
    Ok(format!("GPU Temperature: {}°C", temp))
}

// In server/commands.rs
registry.register(
    "gpuTemp",
    simple_command!(
        "gpuTemp",
        "Get current GPU temperature",
        || screen::gpu_temperature()
    ),
);
```

### Example 2: Command with Validation

```rust
// In screen/mod.rs
pub fn set_brightness(screen: Option<&str>, value: &str) -> Result<(), Box<dyn std::error::Error>> {
    let brightness: u8 = value.parse()
        .map_err(|_| "Brightness must be 0-100")?;
    
    if brightness > 100 {
        return Err("Brightness must be 0-100".into());
    }
    
    // Apply brightness
    apply_brightness(screen, brightness)?;
    Ok(())
}

// In server/commands.rs
registry.register(
    "setBrightness",
    screen_setter_command(
        "Sets screen brightness (0-100)",
        |screen, value| screen::set_brightness(screen, value),
    ),
);
```

### Example 3: Command with Multiple Options

```rust
// In screen/mod.rs
pub fn set_color_profile(profile: &str) -> Result<(), Box<dyn std::error::Error>> {
    let valid_profiles = ["srgb", "adobe-rgb", "dcip3", "rec2020"];
    
    if !valid_profiles.contains(&profile) {
        return Err(format!(
            "Invalid profile. Valid options: {}",
            valid_profiles.join(", ")
        ).into());
    }
    
    apply_color_profile(profile)?;
    Ok(())
}

// In server/commands.rs
registry.register(
    "setColorProfile",
    Box::new(ArgCommand {
        name: "setColorProfile".to_string(),
        description: "Sets color profile (srgb, adobe-rgb, dcip3, rec2020)".to_string(),
        expected_args: 1,
        executor: Box::new(|args| screen::set_color_profile(args[0])),
    }),
);
```

## Implementing Custom Handlers

For complex cases, you can implement the `CommandHandler` trait:

```rust
struct ComplexCommand {
    description: String,
    // ... additional fields ...
}

impl CommandHandler for ComplexCommand {
    fn execute(&self, args: &[&str]) -> CommandResult {
        // Complex execution logic
        match self.process_args(args) {
            Ok(result) => Ok(result),
            Err(e) => Err(CommandError::ExecutionError(e)),
        }
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn expected_args(&self) -> Option<usize> {
        None // Accepts any number of arguments
    }
}

// Register
registry.register(
    "complexCommand",
    Box::new(ComplexCommand {
        description: "A complex command handler".to_string(),
    }),
);
```
