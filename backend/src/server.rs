// Contains functions like start_minecraft_server() that use tokio::process::Command to spawn and manage the Minecraft server process asynchronously.

use std::io::Result;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::UnboundedSender;

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
        // let mut command = Command::new("R:\\GameServers\\minecraftNeoforge1.21.1\\run.bat");
        let mut command = Command::new("/home/roanm/Downloads/Server/run.sh");
        // Set the working directory here.
        command.current_dir("/home/roanm/Downloads/Server");
        // Optionally add any necessary command-line arguments:
        // command.arg("some_arg");

        // Set up the process to capture stdout and stderr.
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

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
            // Attempt to gracefully shut down the server by sending "stop\n"
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(b"stop\n").await?;
                stdin.flush().await?;
            } else {
                // If for some reason stdin is not available, fallback to killing the process.
                child.kill().await?;
            }
            // Wait for the server process to exit gracefully.
            child.wait().await?;
            self.child = None;
        }
        Ok(())
    }

    /// Checks if the Minecraft server process is currently running.
    pub fn is_running(&self) -> bool {
        self.child.is_some()
    }

    /// Sends a command to the Minecraft server console.
    ///
    /// # Arguments
    /// * `command` - The command to send to the server
    ///
    /// # Returns
    /// * `Ok(())` if the command was sent successfully
    /// * `Err` if there was an error sending the command
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
