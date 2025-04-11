import { useEffect, useRef, useState } from "react";

interface WebSocketOptions {
	url: string;
}

export function useWebSocket({ url }: WebSocketOptions) {
	const ws = useRef<WebSocket | null>(null);
	const [messages, setMessages] = useState<string[]>([]);
	const [connected, setConnected] = useState(false);

	useEffect(() => {
		// Use absolute WebSocket URL with the backend port
		const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
		const wsUrl = url.startsWith('ws') ? url : `${wsProtocol}//localhost:8080${url}`;
		ws.current = new WebSocket(wsUrl);

		ws.current.onopen = () => {
			console.log("WebSocket connected");
			setConnected(true);
		};

		ws.current.onmessage = (event) => {
			setMessages((prev) => [...prev, event.data]);
		};

		ws.current.onerror = (error) => {
			console.error("WebSocket error:", error);
		};

		ws.current.onclose = () => {
			console.log("WebSocket closed");
			setConnected(false);
		};

		// Cleanup on unmount.
		return () => {
			ws.current?.close();
		};
	}, [url]);

	const sendMessage = (message: string) => {
		if (ws.current && connected) {
			ws.current.send(message);
		} else {
			console.error("WebSocket is not connected");
		}
	};

	return { messages, sendMessage, connected };
}
