//! Regmsg Shell - Interactive command line interface for regmsgd
//!
//! This binary provides an interactive shell to communicate with the regmsgd daemon
//! using ZeroMQ. It allows sending commands and receiving responses in a user-friendly format.

use std::io::{self, Write};
use std::time::Instant;

use clap::Parser;
use zmq;

/// Default ZeroMQ endpoint for regmsgd
const ENDPOINT_DEFAULT: &str = "ipc:///var/run/regmsgd.sock";
/// Default timeout for requests in milliseconds
const TIMEOUT: i32 = 5000;

/// Regmsg Shell - CLI for regmsgd
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// ZeroMQ endpoint
    #[arg(long, default_value = ENDPOINT_DEFAULT)]
    endpoint: String,

    /// Request timeout in milliseconds
    #[arg(long, default_value_t = TIMEOUT)]
    timeout: i32,
}

struct RegmsgShell {
    #[allow(dead_code)] // Kept for connection reuse and settings
    endpoint: String,
    #[allow(dead_code)] // Kept for connection reuse and settings
    timeout: i32,
    #[allow(dead_code)] // Context needs to be kept alive for the connection
    context: zmq::Context,
    socket: zmq::Socket,
}

impl RegmsgShell {
    /// Initialize the RegmsgShell with connection parameters
    fn new(endpoint: String, timeout: i32) -> Result<Self, Box<dyn std::error::Error>> {
        let context = zmq::Context::new();
        let socket = context.socket(zmq::REQ)?;

        // Set socket timeouts to prevent hanging
        socket.set_rcvtimeo(timeout)?;
        socket.set_sndtimeo(timeout)?;

        // Connect to the endpoint
        socket.connect(&endpoint)?;

        Ok(RegmsgShell {
            endpoint,
            timeout,
            context,
            socket,
        })
    }

    /// Connect to the regmsgd daemon with error handling
    fn connect(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        // Test the connection by sending a simple command
        if let Err(e) = self.socket.send("listCommands", 0) {
            eprintln!("🔴 Connection failed: {}", e);
            eprintln!(
                "💡 Make sure regmsgd daemon is running and accessible at {}",
                self.endpoint
            );
            return Ok(false);
        }

        match self.socket.recv_string(0) {
            Ok(_) => Ok(true),
            Err(e) => {
                eprintln!("🔴 Connection failed: {}", e);
                eprintln!(
                    "💡 Make sure regmsgd daemon is running and accessible at {}",
                    self.endpoint
                );
                Ok(false)
            }
        }
    }

    /// Format and display the response from the daemon
    fn format_response(&self, reply: &[u8], start_time: Option<Instant>) {
        if let Some(start_time) = start_time {
            let elapsed = start_time.elapsed();
            println!("⏱️  Response time: {:.3}s", elapsed.as_secs_f64());
        }

        // Decode as UTF-8 text, replace invalid sequences with
        let text = String::from_utf8_lossy(reply);

        // Check if response indicates an error
        if text.starts_with("Error:") || text.starts_with("Err:") {
            eprintln!("{}", text);
        } else {
            // Try to parse as JSON and pretty-print if possible
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&text) {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| text.to_string())
                );
            } else {
                // If not JSON, print as-is (without the "🟢 Response:" header)
                println!("{}", text);
            }
        }
    }

    /// Show help information
    fn show_help(&self) {
        let help_text = r#"
🎮 Regmsg Shell Help
====================
Shell Commands:
  help            - Show this help message
  clear           - Clear the screen
  exit/quit/q     - Exit the shell

Daemon Commands:
  listCommands    - Show all available commands from regmsgd
  [other commands] - All other commands are sent to the regmsgd daemon
"#;
        println!("{}", help_text);
    }

    /// Handle internal shell commands
    fn handle_internal_command(&mut self, cmd: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let cmd_lower = cmd.trim().to_lowercase();

        match cmd_lower.as_str() {
            "exit" | "quit" | "q" => {
                return Ok(true); // Signal to exit
            }
            "help" | "h" | "?" => {
                self.show_help();
                return Ok(true); // Command handled, don't send to daemon
            }
            "clear" => {
                // Clear screen using ANSI escape codes
                print!("\x1B[2J\x1B[1;1H");
                return Ok(true); // Command handled, don't send to daemon
            }
            _ => {
                // Not an internal command, continue with normal processing
            }
        }

        Ok(false)
    }

    /// Run the main interactive command loop
    fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Use 'help' for shell commands.");
        println!("Enter a command or 'exit' to quit.");
        println!("💡 Daemon commands: Use 'listCommands' to see available commands from regmsgd.");
        println!("{}", "─".repeat(80));

        loop {
            // Show hostname in prompt
            let hostname = get_hostname();
            let prompt = format!("[{}]> ", hostname);

            // Print the prompt and flush to ensure it appears immediately
            print!("{}", prompt);
            io::stdout().flush()?;

            // Read a line of input from the user
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("\n🔴 Error reading input: {}", e);
                    continue;
                }
            }

            // Remove trailing newline
            let cmd = input.trim();

            if cmd.is_empty() {
                continue;
            }

            // Check if it's an internal command
            match self.handle_internal_command(cmd)? {
                true => {
                    // Command was handled internally (exit, help, clear, etc.)
                    // Check if it was an exit command
                    let cmd_lower = cmd.trim().to_lowercase();
                    if cmd_lower == "exit" || cmd_lower == "quit" || cmd_lower == "q" {
                        break; // Exit the shell
                    }
                    // For other internal commands (help, clear), just continue to next iteration
                    continue;
                }
                false => {
                    // Command was not handled internally, send to daemon
                }
            }

            // Record start time to measure response time
            let start_time = Some(Instant::now());

            // Send command to the daemon
            match self.socket.send(cmd, 0) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("🔴 Error sending command: {}", e);
                    eprintln!("💡 Check your connection or command syntax");
                    continue;
                }
            }

            // Receive response from the daemon
            match self.socket.recv_bytes(0) {
                Ok(reply) => {
                    self.format_response(&reply, start_time);
                }
                Err(e) => {
                    eprintln!("🔴 Error receiving response: {}", e);
                    eprintln!(
                        "💡 The command may have timed out or the daemon may be unresponsive"
                    );
                }
            }

            println!("{}", "-".repeat(80));
        }

        Ok(())
    }
}

/// Get the hostname of the current machine
fn get_hostname() -> String {
    match hostname::get() {
        Ok(name) => name.to_string_lossy().to_string(),
        Err(_) => "localhost".to_string(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut shell = RegmsgShell::new(args.endpoint, args.timeout)?;

    if !shell.connect()? {
        eprintln!("🔴 Failed to connect to regmsgd. The daemon may not be running.");
        std::process::exit(1);
    }

    shell.run()?;

    Ok(())
}
