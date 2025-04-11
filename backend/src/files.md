# Module Documentation

The backend is split into several core modules that work together to provide a web interface for managing a Minecraft server:

## main.rs
The entry point of the application that:
- Sets up the Actix-web server
- Initializes shared state
- Creates communication channels for server logs
- Configures and binds HTTP routes
- Starts the web server on localhost:8080

## routes.rs
Defines all HTTP endpoints and their handlers including:
- `/start` - POST endpoint to start the Minecraft server
- `/stop` - POST endpoint to stop the Minecraft server 
- `/status` - GET endpoint to check server status
- `/ws` - WebSocket endpoint for real-time console access

## server.rs
Contains the core Minecraft server management logic:
- `MinecraftServer` struct that wraps the actual server process
- Handles starting/stopping the server process asynchronously
- Captures and forwards server stdout/stderr output
- Manages the server's lifecycle

## state.rs
Manages shared application state:
- `AppState` struct containing shared server state
- Handles server status tracking
- Manages log message broadcasting
- Provides methods to start/stop/check server status
- Creates log receivers for WebSocket connections

## websocket.rs
Implements WebSocket functionality for real-time console access:
- `ConsoleWebSocket` actor to handle WebSocket connections
- Implements heartbeat monitoring to maintain connections
- Handles incoming WebSocket messages (future: server commands)
- Manages WebSocket lifecycle (connect/disconnect)
- Will eventually allow bidirectional communication with server console

The architecture uses Actix-web for HTTP/WebSocket handling and Tokio for asynchronous process management, 
providing a robust foundation for managing a Minecraft server through a web interface.
