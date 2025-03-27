// Implements WebSocket handlers using Actix-web actors. This file will include logic to stream server logs to connected clients and accept command input.
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::{ws, Actor};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::state::AppState;

/// WebSocket connection actor
pub struct WebSocketConnection {
    app_state: web::Data<Arc<Mutex<AppState>>>,
}

impl Actor for WebSocketConnection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // When a new WebSocket connection is established, set up log forwarding
        let log_receiver = self.app_state.lock().unwrap().create_log_receiver();
        self.handle_logs(log_receiver, ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                // Handle incoming commands from the WebSocket client
                if let Ok(mut state) = self.app_state.lock() {
                    // TODO: Implement command handling
                    // For now, just echo back the message
                    ctx.text(format!("Received command: {}", text));
                }
            }
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

impl WebSocketConnection {
    pub fn new(app_state: web::Data<Arc<Mutex<AppState>>>) -> Self {
        WebSocketConnection { app_state }
    }

    fn handle_logs(
        &self,
        mut receiver: UnboundedReceiver<String>,
        ctx: &mut ws::WebsocketContext<Self>,
    ) {
        // Clone necessary data for the async block
        let handle = ctx.address();

        // Spawn a task to forward log messages to the WebSocket
        actix::spawn(async move {
            while let Some(log) = receiver.recv().await {
                // Send the log message to the WebSocket client
                handle.do_send(ws::Message::Text(log));
            }
        });
    }
}

/// WebSocket handshake and connection establishment
pub async fn ws_index(
    req: HttpRequest,
    stream: web::Payload,
    app_state: web::Data<Arc<Mutex<AppState>>>,
) -> Result<HttpResponse, Error> {
    ws::start(WebSocketConnection::new(app_state), &req, stream)
}
