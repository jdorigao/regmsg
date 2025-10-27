//! Configuration Module
//!
//! This module provides constants and default configurations for the regmsg daemon.

/// Constants for default settings
pub const DEFAULT_SOCKET_PATH: &str = "/var/run/regmsgd.sock";
pub const DEFAULT_SCREENSHOT_DIR: &str = "/userdata/screenshots";
pub const DEFAULT_MAX_RESOLUTION: &str = "1920x1080";
//pub const DEFAULT_LOG_PATH: &str = "/var/log/regmsg.log";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_SOCKET_PATH, "/var/run/regmsgd.sock");
        assert_eq!(DEFAULT_SCREENSHOT_DIR, "/userdata/screenshots");
        assert_eq!(DEFAULT_MAX_RESOLUTION, "1920x1080");
        //assert_eq!(DEFAULT_LOG_PATH, "/var/log/regmsg.log");
    }
}
