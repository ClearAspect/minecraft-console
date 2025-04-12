import { useEffect, useRef, useState, useCallback } from "react";

interface WebSocketOptions {
	url: string;
	reconnectInterval?: number;  // milliseconds
	maxReconnectAttempts?: number;
}

// Define a true singleton WebSocket manager
class WebSocketManager {
	private static instance: WebSocketManager | null = null;
	private socket: WebSocket | null = null;
	private url: string = '';
	private connectionAttemptInProgress: boolean = false;
	private intentionalClose: boolean = false;
	private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
	private reconnectCount: number = 0;
	private maxReconnectAttempts: number = 5;
	private reconnectInterval: number = 3000;
	private subscribers = new Map<number, (message: string) => void>();
	private connectionStateSubscribers = new Map<number, (connected: boolean) => void>();
	private nextSubscriberId: number = 1;
	private connectPromise: Promise<void> | null = null;
	private connectResolve: (() => void) | null = null;

	// Get singleton instance
	public static getInstance(): WebSocketManager {
		if (!WebSocketManager.instance) {
			WebSocketManager.instance = new WebSocketManager();
		}
		return WebSocketManager.instance;
	}

	// Configure WebSocket settings
	public configure(options: WebSocketOptions): void {
		this.url = options.url;
		this.maxReconnectAttempts = options.maxReconnectAttempts || 5;
		this.reconnectInterval = options.reconnectInterval || 3000;
	}

	// Connect to WebSocket server
	public connect(): Promise<void> {
		// If we're already connected or connecting, return existing promise
		if (this.socket && (this.socket.readyState === WebSocket.OPEN || this.socket.readyState === WebSocket.CONNECTING)) {
			console.log("WebSocket: Already connected or connecting");
			return Promise.resolve();
		}

		// If connection attempt already in progress, return the existing promise
		if (this.connectionAttemptInProgress && this.connectPromise) {
			console.log("WebSocket: Connection attempt in progress, returning existing promise");
			return this.connectPromise;
		}

		// Create a new connection promise
		this.connectPromise = new Promise<void>((resolve) => {
			this.connectResolve = resolve;
		});

		this.connectionAttemptInProgress = true;
		this.intentionalClose = false;

		// Close existing socket if it exists
		if (this.socket) {
			this.intentionalClose = true;
			this.socket.close();
			this.socket = null;
		}

		// Calculate wsUrl
		const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
		const wsUrl = this.url.startsWith('ws') ? this.url : `${wsProtocol}//localhost:8080${this.url}`;

		console.log(`WebSocket: Creating new connection to ${wsUrl}`);

		try {
			// Create new WebSocket
			this.socket = new WebSocket(wsUrl);

			// Set up event handlers
			this.socket.onopen = this.handleOpen.bind(this);
			this.socket.onmessage = this.handleMessage.bind(this);
			this.socket.onclose = this.handleClose.bind(this);
			this.socket.onerror = this.handleError.bind(this);
		} catch (error) {
			console.error("WebSocket: Error creating connection", error);
			this.connectionAttemptInProgress = false;
			this.scheduleReconnect();
		}

		return this.connectPromise;
	}

	// Close WebSocket connection
	public close(): void {
		if (this.socket) {
			this.intentionalClose = true;
			this.socket.close();
			this.socket = null;
		}

		// Clear any reconnect timer
		if (this.reconnectTimer) {
			clearTimeout(this.reconnectTimer);
			this.reconnectTimer = null;
		}
	}

	// Send message through WebSocket
	public send(message: string): boolean {
		if (this.socket && this.socket.readyState === WebSocket.OPEN) {
			this.socket.send(message);
			return true;
		}
		return false;
	}

	// Subscribe to messages
	public subscribeToMessages(callback: (message: string) => void): number {
		const id = this.nextSubscriberId++;
		this.subscribers.set(id, callback);
		return id;
	}

	// Unsubscribe from messages
	public unsubscribeFromMessages(id: number): void {
		this.subscribers.delete(id);
	}

	// Subscribe to connection state changes
	public subscribeToConnectionState(callback: (connected: boolean) => void): number {
		const id = this.nextSubscriberId++;
		this.connectionStateSubscribers.set(id, callback);

		// Immediately notify about current state
		if (this.socket) {
			callback(this.socket.readyState === WebSocket.OPEN);
		} else {
			callback(false);
		}

		return id;
	}

	// Unsubscribe from connection state changes
	public unsubscribeFromConnectionState(id: number): void {
		this.connectionStateSubscribers.delete(id);
	}

	// Get connection state
	public isConnected(): boolean {
		return !!(this.socket && this.socket.readyState === WebSocket.OPEN);
	}

	// Get connection attempt state
	public isConnecting(): boolean {
		return this.connectionAttemptInProgress;
	}

	// Reset state - mainly for testing
	public reset(): void {
		this.close();
		this.subscribers.clear();
		this.connectionStateSubscribers.clear();
		this.reconnectCount = 0;
		this.connectionAttemptInProgress = false;
		this.intentionalClose = false;
	}

	// PRIVATE METHODS

	// Handle WebSocket open event
	private handleOpen(): void {
		console.log("WebSocket: Connection established");
		this.connectionAttemptInProgress = false;
		this.reconnectCount = 0;

		// Notify connection state subscribers
		this.connectionStateSubscribers.forEach(callback => {
			callback(true);
		});

		// Broadcast welcome message to subscribers
		this.subscribers.forEach(callback => {
			callback("Connected to server console");
		});

		// Resolve the connect promise
		if (this.connectResolve) {
			this.connectResolve();
			this.connectResolve = null;
		}
	}

	// Handle WebSocket message event
	private handleMessage(event: MessageEvent): void {
		// Broadcast the message to all subscribers
		this.subscribers.forEach(callback => {
			callback(event.data);
		});
	}

	// Handle WebSocket close event
	private handleClose(event: CloseEvent): void {
		console.log(`WebSocket: Connection closed (code: ${event.code}, reason: ${event.reason || 'No reason provided'})`);
		this.connectionAttemptInProgress = false;

		// Notify connection state subscribers
		this.connectionStateSubscribers.forEach(callback => {
			callback(false);
		});

		// Schedule reconnect if close was not intentional
		if (!this.intentionalClose) {
			this.scheduleReconnect();
		}

		this.socket = null;
	}

	// Handle WebSocket error event
	private handleError(event: Event): void {
		console.error("WebSocket: Error occurred", event);
	}

	// Schedule reconnection attempt
	private scheduleReconnect(): void {
		if (this.reconnectCount >= this.maxReconnectAttempts) {
			console.log("WebSocket: Max reconnect attempts reached");
			return;
		}

		// Clear any existing reconnect timer
		if (this.reconnectTimer) {
			clearTimeout(this.reconnectTimer);
		}

		// Calculate backoff time
		const backoffTime = this.reconnectInterval * Math.pow(2, this.reconnectCount);
		console.log(`WebSocket: Attempting reconnect ${this.reconnectCount + 1}/${this.maxReconnectAttempts} in ${backoffTime}ms`);

		// Set timer for reconnect
		this.reconnectTimer = setTimeout(() => {
			this.reconnectCount++;
			this.connect();
		}, backoffTime);
	}
}

// The hook that provides WebSocket functionality to components
export function useWebSocket({
	url,
	reconnectInterval = 3000,
	maxReconnectAttempts = 5
}: WebSocketOptions) {
	const [messages, setMessages] = useState<string[]>([]);
	const [connected, setConnected] = useState(false);
	const wsManager = WebSocketManager.getInstance();
	const messageSubscriptionId = useRef<number | null>(null);
	const connectionSubscriptionId = useRef<number | null>(null);
	const componentMounted = useRef(true);

	// Initialize on first render
	useEffect(() => {
		// Configure the WebSocket manager
		wsManager.configure({
			url,
			reconnectInterval,
			maxReconnectAttempts
		});

		// Subscribe to messages
		messageSubscriptionId.current = wsManager.subscribeToMessages((message) => {
			if (componentMounted.current) {
				setMessages(prev => [...prev, message]);
			}
		});

		// Subscribe to connection state changes
		connectionSubscriptionId.current = wsManager.subscribeToConnectionState((isConnected) => {
			if (componentMounted.current) {
				setConnected(isConnected);
			}
		});

		// Connect to the WebSocket server
		wsManager.connect();

		// Cleanup on unmount
		return () => {
			componentMounted.current = false;

			// Unsubscribe from messages
			if (messageSubscriptionId.current !== null) {
				wsManager.unsubscribeFromMessages(messageSubscriptionId.current);
			}

			// Unsubscribe from connection state
			if (connectionSubscriptionId.current !== null) {
				wsManager.unsubscribeFromConnectionState(connectionSubscriptionId.current);
			}
		};
	}, [url, reconnectInterval, maxReconnectAttempts]);

	// Send message function
	const sendMessage = useCallback((message: string) => {
		if (!wsManager.isConnected()) {
			console.warn("WebSocket: Not connected, attempting to connect before sending");
			if (componentMounted.current) {
				setMessages(prev => [...prev, "Error: WebSocket is not connected, attempting to reconnect..."]);
			}

			// Try to connect and then send the message
			wsManager.connect().then(() => {
				const sent = wsManager.send(message);
				if (!sent && componentMounted.current) {
					setMessages(prev => [...prev, "Error: Failed to send message after reconnect attempt"]);
				}
			});

			return;
		}

		// Send the message if connected
		wsManager.send(message);
	}, []);

	return { messages, sendMessage, connected };
}
