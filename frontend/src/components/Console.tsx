import React, { useState } from "react";
import { useWebSocket } from "../hooks/useWebSocket";

const Console: React.FC = () => {
	// Update the URL if needed.
	const { messages, sendMessage, connected } = useWebSocket({
		url: "/ws",
	});

	const [command, setCommand] = useState("");

	const handleSendCommand = () => {
		if (command.trim() !== "") {
			sendMessage(command.trim());
			setCommand("");
		}
	};

	return (
		<div style={{ marginTop: "20px" }}>
			<h2>Server Console</h2>
			<div
				style={{
					height: "300px",
					overflowY: "auto",
					backgroundColor: "#111",
					color: "#0f0",
					padding: "10px",
					fontFamily: "monospace",
				}}
			>
				{messages.map((msg, idx) => (
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
					style={{ width: "80%" }}
				/>
				<button onClick={handleSendCommand}>Send</button>
			</div>
			<div>
				<small>WebSocket status: {connected ? "Connected" : "Disconnected"}</small>
			</div>
		</div>
	);
};

export default Console;
