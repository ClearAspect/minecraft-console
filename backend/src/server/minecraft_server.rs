//! Implementation of the Minecraft server process management.
//!
//! This file contains the implementation of the MinecraftServer struct
//! that handles starting, stopping, and interacting with the Minecraft
//! server process using Tokio's async process handling.

use std::io::Result;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::UnboundedSender;

/// Represents the Minecraft server process.
///
/// This struct manages the lifecycle of the Minecraft server process including:
/// - Starting the server
/// - Stopping the server
/// - Sending commands to the server
/// - Capturing and forwarding server output
pub struct MinecraftServer {
    /// The child process running the Minecraft server, None if not running.
    child: Option<tokio::process::Child>,
    /// Channel sender to forward log messages to other parts of the application.
    pub log_sender: UnboundedSender<String>,
}

impl MinecraftServer {
    /// Starts the Minecraft server process asynchronously.
    ///
    /// This function:
    /// 1. Spawns the server process
    /// 2. Sets up stdout and stderr redirection
    /// 3. Creates tasks to capture and forward the log output
    ///
    /// # Arguments
    /// * `log_sender` - Channel sender to forward log messages
    ///
    /// # Returns
    /// * `Result<Self>` - New MinecraftServer instance or IO error
    pub async fn start(log_sender: UnboundedSender<String>) -> Result<Self> {
        // Create the command for the server executable
        let mut command = Command::new(r#"R:\GameServers\may25minecraftNeoforge1.21.1\run.bat"#);
        // Set the working directory for the server
        command.current_dir(r#"R:\GameServers\may25minecraftNeoforge1.21.1"#);

        // Configure process I/O streams
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Spawn the server process
        let mut child = command.spawn()?;

        // Set up stdout handling
        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout).lines();
            let sender_clone = log_sender.clone();
            tokio::spawn(async move {
                while let Ok(Some(line)) = reader.next_line().await {
                    // Forward each stdout line to the log channel without duplicate printing
                    if sender_clone.send(line).is_err() {
                        println!("Failed to send stdout log to channel");
                        break;
                    }
                }
                println!("Stdout reader task completed");
            });
        }

        // Set up stderr handling
        if let Some(stderr) = child.stderr.take() {
            let mut reader = BufReader::new(stderr).lines();
            let sender_clone = log_sender.clone();
            tokio::spawn(async move {
                while let Ok(Some(line)) = reader.next_line().await {
                    // Prefix stderr lines with "ERROR:" for clarity but don't print duplicates
                    let error_line = format!("ERROR: {}", line);
                    if sender_clone.send(error_line).is_err() {
                        println!("Failed to send stderr log to channel");
                        break;
                    }
                }
                println!("Stderr reader task completed");
            });
        }

        Ok(MinecraftServer {
            child: Some(child),
            log_sender,
        })
    }

    /// Stops the Minecraft server process gracefully.
    ///
    /// First attempts to send a "stop" command to the server via stdin.
    /// If that fails, falls back to killing the process.
    ///
    /// # Returns
    /// * `Result<()>` - Success or IO error
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(child) = &mut self.child {
            // Attempt to gracefully shut down the server by sending "stop\n"
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(b"stop\n").await?;
                stdin.flush().await?;
            } else {
                // Fallback to killing the process if stdin is not available
                child.kill().await?;
            }
            // Wait for the server process to exit
            child.wait().await?;
            self.child = None;
        }
        Ok(())
    }

    /// Checks if the Minecraft server process is currently running.
    ///
    /// # Returns
    /// * `bool` - True if the server is running, false otherwise
    pub fn is_running(&self) -> bool {
        self.child.is_some()
    }

    /// Sends a command to the Minecraft server console.
    ///
    /// # Arguments
    /// * `command` - The command to send to the server
    ///
    /// # Returns
    /// * `Result<()>` - Success or IO error
    pub async fn send_command(&mut self, command: &str) -> Result<()> {
        if let Some(child) = &mut self.child {
            if let Some(stdin) = child.stdin.as_mut() {
                // Append newline to ensure command is executed
                stdin.write_all(format!("{}\n", command).as_bytes()).await?;
                stdin.flush().await?;
                return Ok(());
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            "Server is not running or stdin is not available",
        ))
    }
}
