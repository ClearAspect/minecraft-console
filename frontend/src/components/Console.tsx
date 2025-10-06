import React, { useState, useRef, useEffect } from "react";
import { useWebSocket } from "../hooks/useWebSocket";

const Console: React.FC = () => {

	// Use singleton WebSocket manager through the hook
	const { messages, sendMessage, connected } = useWebSocket({
		url: "/ws",
		reconnectInterval: 3000,
		maxReconnectAttempts: 5,
	});

	const [command, setCommand] = useState("");
	const consoleRef = useRef<HTMLDivElement>(null);


	// Auto-scroll to bottom when new messages arrive
	useEffect(() => {
		if (consoleRef.current) {
			consoleRef.current.scrollTop = consoleRef.current.scrollHeight;
		}
	}, [messages]);

	const handleSendCommand = () => {
		if (command.trim() !== "") {
			sendMessage(command.trim());
			setCommand("");
		}
	};

	// More aggressive filtering of connection messages
	const filteredMessages = messages.reduce((acc: string[], message: string, index: number) => {
		// Skip all "Connected to Minecraft console WebSocket" messages except the first one
		if (message.includes("Connected to Minecraft console WebSocket")) {
			// Only keep the first connection message
			const previousConnections = acc.filter(m => m.includes("Connected to Minecraft console WebSocket"));
			if (previousConnections.length > 0) {
				return acc;
			}
		}

		// Skip duplicate connection/disconnection messages
		if (index > 0) {
			if (message === "Connected to server console" && messages[index - 1] === "Disconnected from server console") {
				return acc;
			}
			if (message === "Disconnected from server console" &&
				acc.length > 0 &&
				acc[acc.length - 1].includes("Connected to Minecraft console WebSocket")) {
				return acc;
			}
		}

		return [...acc, message];
	}, []);

	return (
		<div style={{ marginTop: "20px" }}>
			<h2>Server Console</h2>
			<div
				ref={consoleRef}
				style={{
					height: "300px",
					overflowY: "auto",
					backgroundColor: "#111",
					color: "#0f0",
					padding: "10px",
					fontFamily: "monospace",
				}}
			>
				{filteredMessages.map((msg, idx) => (
					<div key={idx}>{msg}</div>
				))}
			</div>
			<div style={{ marginTop: "10px" }}>
				<input
					type="text"
					placeholder="Enter command..."
					value={command}
					onChange={(e) => setCommand(e.target.value)}
					onKeyDown={(e) => {
						if (e.key === "Enter") handleSendCommand();
					}}
					disabled={!connected}
					style={{
						width: "80%",
						opacity: connected ? 1 : 0.5
					}}
				/>
				<button
					onClick={handleSendCommand}
					disabled={!connected}
					style={{
						opacity: connected ? 1 : 0.5
					}}
				>
					Send
				</button>
			</div>
			<div style={{
				margin: "10px 0",
				color: connected ? "green" : "red",
				fontWeight: "bold"
			}}>
				WebSocket status: {connected ? "Connected" : "Disconnected"}
			</div>
		</div>
	);
};

export default Console;
