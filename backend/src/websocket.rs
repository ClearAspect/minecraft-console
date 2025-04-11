// src/websocket.rs

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

/// ConsoleWebSocket actor which will handle WebSocket messages.
/// In a more advanced setup, you could include references to shared state
/// so that messages from the Minecraft server can be pushed to clients and
/// commands from clients can be sent to the server.
pub struct ConsoleWebSocket {
    /// The last time the heartbeat was received.
    last_heartbeat: Instant,
    // Shared application state
    app_state: web::Data<Arc<Mutex<AppState>>>,
}

impl ConsoleWebSocket {
    /// Creates a new instance of the ConsoleWebSocket actor.
    pub fn new(app_state: web::Data<Arc<Mutex<AppState>>>) -> Self {
        Self {
            last_heartbeat: Instant::now(),
            app_state,
        }
    }

    /// Helper function that schedules heartbeat pings to ensure the client stays connected.
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

/// Make ConsoleWebSocket an actor.
impl Actor for ConsoleWebSocket {
    type Context = ws::WebsocketContext<Self>;

    /// Called when the actor is started. Begin heartbeat checks.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

/// Handle the stream of WebSocket messages.
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
                // Clone what we need to move into the future
                let text_clone = text.clone();
                let app_state = self.app_state.clone();

                // Spawn the async operation
                actix::spawn(async move {
                    if let Ok(mut state) = app_state.lock() {
                        match state.send_command(&text_clone).await {
                            Ok(_) => {
                                // Note: Since we're in a separate task, we can't use ctx directly
                                // You might want to implement a proper message passing system here
                                println!("Command sent: {}", text_clone);
                            }
                            Err(e) => {
                                println!("Error sending command: {}", e);
                            }
                        }
                    } else {
                        println!("Error: Could not access server state");
                    }
                });

                // Immediately acknowledge receipt of the command
                ctx.text(format!("Command received: {}", text));
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
pub async fn ws_index(
    req: HttpRequest,
    stream: web::Payload,
    app_state: web::Data<Arc<Mutex<AppState>>>,
) -> Result<HttpResponse, Error> {
    ws::start(ConsoleWebSocket::new(app_state), &req, stream)
}
