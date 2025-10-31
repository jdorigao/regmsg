#!/usr/bin/env python3
"""
Regmsg Shell - Interactive command line interface for regmsgd

This script provides an interactive shell to communicate with the regmsgd daemon
using ZeroMQ. It allows sending commands and receiving responses in a user-friendly format.

Usage:
    python regmsg_shell.py
"""

import zmq
import binascii
import sys
import time
import argparse
from datetime import datetime

try:
    import readline  # For command history (Unix/Mac)

    HAS_READLINE = True
except ImportError:
    HAS_READLINE = False

# Default ZeroMQ endpoint for regmsgd
ENDPOINT_DEFAULT = "ipc:///var/run/regmsgd.sock"
# Default timeout for requests in milliseconds
TIMEOUT = 5000


class RegmsgShell:
    """
    An interactive shell for communicating with regmsgd daemon

    This class handles the connection to the regmsgd daemon and provides
    an interactive command line interface for sending commands and receiving responses.
    """

    def __init__(self, endpoint=ENDPOINT_DEFAULT, timeout=TIMEOUT):
        """
        Initialize the RegmsgShell with connection parameters

        Args:
            endpoint (str): ZeroMQ endpoint to connect to
            timeout (int): Request timeout in milliseconds
        """
        self.endpoint = endpoint
        self.timeout = timeout
        self.ctx = None
        self.sock = None
        self.command_history = []

    def connect(self):
        """
        Connect to the regmsgd daemon with error handling

        Returns:
            bool: True if connection was successful, False otherwise
        """
        try:
            # Create ZeroMQ context and socket
            self.ctx = zmq.Context()
            self.sock = self.ctx.socket(zmq.REQ)

            # Set socket timeouts to prevent hanging
            self.sock.setsockopt(zmq.RCVTIMEO, self.timeout)
            self.sock.setsockopt(zmq.SNDTIMEO, self.timeout)

            # Connect to the endpoint
            self.sock.connect(self.endpoint)

            # Test the connection by sending a simple command
            self.sock.send_string("listCommands")
            reply = self.sock.recv()
            print("✅ Connected to regmsgd")
            return True

        except zmq.Again:
            # This happens when the request times out
            print(f"❌ Timeout connecting to {self.endpoint}. Is the daemon running?")
            return False
        except Exception as e:
            # Handle other connection errors
            print(f"❌ Error connecting to regmsgd: {e}")
            return False

    def format_response(self, reply, start_time=None):
        """
        Format and display the response from the daemon

        Args:
            reply (bytes): Raw response from the daemon
            start_time (float, optional): Timestamp when request was sent for timing info
        """
        if start_time:
            elapsed = time.time() - start_time
            print(f"⏱️  Response time: {elapsed:.3f}s")

        try:
            # Try to decode as UTF-8 text
            text = reply.decode("utf-8")
            print("📥 Response (text):")
            print(text)
        except UnicodeDecodeError:
            # If text decoding fails, display as hexadecimal
            print("📥 Response (hex):")
            print(binascii.hexlify(reply).decode())
        except Exception as e:
            # Handle other parsing errors
            print(f"📥 Error parsing response: {e}")

    def run(self):
        """
        Run the main interactive command loop
        """
        # This check ensures self.sock is not None, satisfying static analysis
        if self.sock is None:
            print("❌ Socket is not connected. Cannot run shell.")
            return

        print("Enter a command or 'exit' to quit.")
        print("Use 'listCommands' to see available commands.")
        print("-" * 80)

        while True:
            try:
                import socket

                # Show hostname in prompt
                prompt = f"🎮 [{socket.gethostname()}]> "
                cmd = input(prompt).strip()

                if not cmd:
                    continue
                if cmd.lower() in ("exit", "quit", "q"):
                    break

                # Add to command history if readline is available
                if HAS_READLINE and cmd not in self.command_history:
                    self.command_history.append(cmd)

                # Record start time to measure response time
                start_time = time.time()

                # Send command to the daemon
                try:
                    self.sock.send_string(cmd)
                except zmq.Again:
                    print("❌ Timeout sending command")
                    continue
                except Exception as e:
                    print(f"❌ Error sending command: {e}")
                    break

                # Receive response from the daemon
                try:
                    reply = self.sock.recv()
                    self.format_response(reply, start_time)
                except zmq.Again:
                    print("❌ Timeout waiting for response")
                except Exception as e:
                    print(f"❌ Error receiving response: {e}")
                    break

                print("-" * 80)

            except KeyboardInterrupt:
                # Handle Ctrl+C gracefully
                print("\nUse 'exit' or 'quit' to exit.")
            except EOFError:
                # Handle Ctrl+D (or end of input)
                print("\nExiting...")
                break

    def close(self):
        """
        Close the connection to the daemon
        """
        if self.sock:
            self.sock.close()
        if self.ctx:
            self.ctx.term()


def main():
    """
    Main function to run the regmsg shell with argument parsing
    """
    parser = argparse.ArgumentParser(description="Regmsg Shell - CLI for regmsgd")
    parser.add_argument(
        "--endpoint",
        default=ENDPOINT_DEFAULT,
        help="ZeroMQ endpoint (default: %(default)s)",
    )
    parser.add_argument(
        "--timeout",
        type=int,
        default=TIMEOUT,
        help="Request timeout in milliseconds (default: %(default)s)",
    )

    args = parser.parse_args()

    # Create and initialize the shell
    shell = RegmsgShell(endpoint=args.endpoint, timeout=args.timeout)

    if not shell.connect():
        sys.exit(1)

    try:
        # Run the interactive shell
        shell.run()
    finally:
        # Always close the connection
        shell.close()


if __name__ == "__main__":
    main()
