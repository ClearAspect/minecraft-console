//! Implementation of WebSocket functionality for real-time console access.
//!
//! This file contains the WebSocket actor implementation that handles:
//! - WebSocket connections and disconnections
//! - Heartbeat monitoring to maintain connections
//! - Log message forwarding to clients
//! - Command processing from clients to the server

use actix::prelude::*;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::state::AppState;

/// Heartbeat interval for pings
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// Client timeout duration.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// WebSocket actor for the Minecraft server console.
///
/// This actor:
/// - Maintains the WebSocket connection with clients
/// - Forwards log messages from the server to clients
/// - Processes commands from clients and forwards them to the server
/// - Handles connection lifecycle (connect/disconnect)
pub struct ConsoleWebSocket {
    /// The last time the heartbeat was received.
    last_heartbeat: Instant,
    /// Shared application state
    app_state: web::Data<Arc<Mutex<AppState>>>,
    /// Client ID assigned by AppState
    client_id: usize,
    /// Channel for receiving log messages
    log_rx: Option<tokio::sync::mpsc::UnboundedReceiver<String>>,
}

impl ConsoleWebSocket {
    /// Creates a new instance of the ConsoleWebSocket actor.
    ///
    /// # Arguments
    /// * `app_state` - Shared application state
    ///
    /// # Returns
    /// * New ConsoleWebSocket instance
    pub fn new(app_state: web::Data<Arc<Mutex<AppState>>>) -> Self {
        Self {
            last_heartbeat: Instant::now(),
            app_state,
            client_id: 0,
            log_rx: None,
        }
    }

    /// Schedules heartbeat pings to ensure the client stays connected.
    ///
    /// This function sets up a recurring timer that sends ping messages
    /// to the client and checks for client timeouts.
    ///
    /// # Arguments
    /// * `ctx` - WebSocket context
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |actor, ctx| {
            // Check if the client has timed out.
            if Instant::now().duration_since(actor.last_heartbeat) > CLIENT_TIMEOUT {
                println!("Websocket client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

/// Message type for internal actor communication to forward logs
#[derive(Message)]
#[rtype(result = "()")]
pub struct ForwardLog(String);

/// Handler for ForwardLog messages
impl Handler<ForwardLog> for ConsoleWebSocket {
    type Result = ();

    fn handle(&mut self, msg: ForwardLog, ctx: &mut Self::Context) {
        // Send log message to the WebSocket client
        let log = msg.0;

        // To See if the log is being sent to specific client
        // println!(
        //     "Client {}: Sending log via WebSocket: {}",
        //     self.client_id, &log
        // );
        ctx.text(log);
    }
}

/// WebSocket actor implementation
impl Actor for ConsoleWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Called when the actor is started.
    /// Sets up heartbeat checks and log streaming.
    fn started(&mut self, ctx: &mut Self::Context) {
        // Start heartbeat monitoring
        self.hb(ctx);

        // Register this client and set up log streaming
        if let Ok(mut app_state) = self.app_state.lock() {
            let (client_id, log_rx) = app_state.register_client();
            self.client_id = client_id;

            // Get address of self
            let addr = ctx.address();

            // Format a welcome message with timestamp to help identify separate connections
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            ctx.text(format!(
                "--- Connected to Minecraft console WebSocket (client ID: {}, timestamp: {}) ---",
                client_id, timestamp
            ));

            // Send instruction to help debug multiple connections
            ctx.text("If you see multiple connection messages, check your application for duplicate WebSocket connections");

            // Spawn a task to forward logs to this WebSocket client
            let mut log_rx = log_rx;
            actix::spawn(async move {
                println!("[Log Receiver]: Started (Client {})", client_id);
                while let Some(log) = log_rx.recv().await {
                    println!("[Log Receiver]: Fowarded (Client {}): {}", client_id, &log);

                    // Send the log message to the WebSocket actor
                    addr.do_send(ForwardLog(log));
                }
                println!("[Log Receiver]: Terminated (Client {})", client_id);
            });
        } else {
            ctx.text("[Log Receiver]:  Could not access server state");
            ctx.stop();
        }
    }

    /// Called when the actor is stopping.
    /// Unregisters the client from the application state.
    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // Unregister this client when the WebSocket is closing
        if let Ok(mut app_state) = self.app_state.lock() {
            app_state.unregister_client(self.client_id);
        }
        Running::Stop
    }
}

/// WebSocket message handler implementation
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ConsoleWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                // Reset heartbeat timer and respond with Pong.
                self.last_heartbeat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                // Update heartbeat timer on pong.
                self.last_heartbeat = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                // Only log commands, not debug every received message
                if !text.trim().is_empty() {
                    println!("Client {}: Command received: {}", self.client_id, text);
                }

                // Clone what we need to move into the future
                let text_clone = text.clone();
                let app_state = self.app_state.clone();
                let client_id = self.client_id;

                // Immediately acknowledge receipt of the command
                ctx.text(format!("Command received: {}", text));

                // Spawn the async operation to send command to the server
                actix::spawn(async move {
                    if let Ok(mut state) = app_state.lock() {
                        match state.send_command(&text_clone).await {
                            Ok(_) => {
                                // Command was sent successfully - no need to log
                            }
                            Err(e) => {
                                // Only log errors
                                println!("Client {}: Error sending command: {}", client_id, e);
                            }
                        }
                    } else {
                        println!("Client {}: Error: Could not access server state", client_id);
                    }
                });
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                // Handle connection close requests.
                ctx.close(reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

/// HTTP handler to upgrade incoming requests to WebSocket connections.
///
/// # Arguments
/// * `req` - HTTP request
/// * `stream` - Payload stream
/// * `app_state` - Shared application state
///
/// # Returns
/// * HTTP response or error
pub async fn ws_index(
    req: HttpRequest,
    stream: web::Payload,
    app_state: web::Data<Arc<Mutex<AppState>>>,
) -> Result<HttpResponse, Error> {
    ws::start(ConsoleWebSocket::new(app_state), &req, stream)
}
