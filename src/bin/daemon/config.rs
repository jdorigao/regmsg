//! Configuration Module
//!
//! This module provides constants and default configurations for the regmsg daemon.

/// Constants for default settings
pub const DEFAULT_SOCKET_PATH: &str = "/var/run/regmsgd.sock";
pub const DEFAULT_SCREENSHOT_DIR: &str = "/userdata/screenshots";
pub const DEFAULT_MAX_RESOLUTION: &str = "1920x1080";
pub const DEFAULT_LOG_PATH: &str = "/var/log/regmsg.log";
pub const DEFAULT_SWAYSOCK_PATH: &str = "/var/run/sway-ipc.0.sock";

/// Constants for game controller database paths
pub const GAMECONTROLLER_DB_PATHS: &[&str] = &[
    "/userdata/system/configs/emulationstation/gamecontrollerdb.txt",
    "/usr/share/emulationstation/gamecontrollerdb.txt",
];
