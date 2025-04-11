import React from "react";

interface StatusIndicatorProps {
	status: string;
}

const StatusIndicator: React.FC<StatusIndicatorProps> = ({ status }) => {
	// First log the exact status for debugging
	console.log("StatusIndicator received status:", status);

	// Determine the server state
	const isRunning = status.toLowerCase().includes("running");
	const isUnknown = status.toLowerCase() === "unknown";
	const isOffline = status.toLowerCase().includes("not running") ||
		status.toLowerCase().includes("offline") ||
		status.toLowerCase() === "";

	const style = {
		color: isRunning ? "green" : isUnknown ? "orange" : "red",
		fontWeight: "bold",
		padding: "5px",
		margin: "10px 0",
		borderRadius: "4px",
		display: "inline-block",
	};

	return (
		<div>
			<div style={style}>Status: {status}</div>
			<div style={{ fontSize: "0.8em", marginTop: "5px" }}>
				Last updated: {new Date().toLocaleTimeString()}
			</div>
		</div>
	);
};

export default StatusIndicator;
