// Contains functions like start_minecraft_server() that use tokio::process::Command to spawn and manage the Minecraft server process asynchronously.

use std::io::Result;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

/// Struct representing the Minecraft server process.
pub struct MinecraftServer {
    /// The child process running the Minecraft server.
    child: Option<tokio::process::Child>,
    /// A channel sender to forward log messages to other parts of your application.
    pub log_sender: UnboundedSender<String>,
}

impl MinecraftServer {
    /// Starts the Minecraft server process asynchronously.
    ///
    /// This function spawns the process, redirects its stdout and stderr, and spawns tasks
    /// to capture and forward the log output.
    pub async fn start(log_sender: UnboundedSender<String>) -> Result<Self> {
        // Replace the path and any arguments with your server's executable details.
        // TODO
        let mut command = Command::new("R:\\GameServers\\minecraftNeoforge1.21.1\\run.bat");
        // Optionally add any necessary command-line arguments:
        // command.arg("some_arg");

        // Set up the process to capture stdout and stderr.
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        // Spawn the process.
        let mut child = command.spawn()?;

        // Handle stdout.
        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout).lines();
            let sender_clone = log_sender.clone();
            tokio::spawn(async move {
                while let Ok(Some(line)) = reader.next_line().await {
                    // Forward each stdout line to the log channel.
                    let _ = sender_clone.send(line);
                }
            });
        }

        // Handle stderr.
        if let Some(stderr) = child.stderr.take() {
            let mut reader = BufReader::new(stderr).lines();
            let sender_clone = log_sender.clone();
            tokio::spawn(async move {
                while let Ok(Some(line)) = reader.next_line().await {
                    // Prefix stderr lines with "ERROR:" for clarity.
                    let _ = sender_clone.send(format!("ERROR: {}", line));
                }
            });
        }

        Ok(MinecraftServer {
            child: Some(child),
            log_sender,
        })
    }

    /// Stops the Minecraft server process gracefully.
    ///
    /// On Windows, this example simply kills the process. You might adjust the logic if you
    /// have a more graceful shutdown command.
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(child) = &mut self.child {
            // Attempt to kill the process.
            child.kill().await?;
            // Wait for the process to exit.
            child.wait().await?;
            self.child = None;
        }
        Ok(())
    }

    /// Checks if the Minecraft server process is currently running.
    pub fn is_running(&self) -> bool {
        self.child.is_some()
    }
}
