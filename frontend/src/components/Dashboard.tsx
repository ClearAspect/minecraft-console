import React, { useState, useEffect } from "react";
import { startServer, stopServer, fetchStatus } from "../utils/api";
import Console from "./Console";
import StatusIndicator from "./StatusIndicator";

const Dashboard: React.FC = () => {
	const [status, setStatus] = useState("unknown");
	const [message, setMessage] = useState("");
	const [isLoading, setIsLoading] = useState(false);
	const [error, setError] = useState<string | null>(null);
	const [filePath, setFilePath] = useState("");

	const handleStart = async () => {
		setIsLoading(true);
		setError(null);
		try {
			const res = await startServer(filePath);
			setMessage(res);
			await updateStatus();
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'An unknown error occurred';
			setError(errorMessage);
			setMessage(`Failed to start server: ${errorMessage}`);
		} finally {
			setIsLoading(false);
		}
	};

	const handleStop = async () => {
		setIsLoading(true);
		setError(null);
		try {
			const res = await stopServer();
			setMessage(res);
			await updateStatus();
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'An unknown error occurred';
			setError(errorMessage);
			setMessage(`Failed to stop server: ${errorMessage}`);
		} finally {
			setIsLoading(false);
		}
	};

	const updateStatus = async () => {
		try {
			const stat = await fetchStatus();
			setStatus(stat);
			setError(null);
		} catch (error) {
			const errorMessage = error instanceof Error ? error.message : 'Network error';
			setStatus("offline");
			setError(`Error fetching status: ${errorMessage}`);
		}
	};

	// Refresh the status every 5 seconds
	useEffect(() => {
		let mounted = true;

		const fetchData = async () => {
			if (mounted) {
				await updateStatus();
			}
		};

		fetchData();
		const interval = setInterval(fetchData, 5000);

		return () => {
			mounted = false;
			clearInterval(interval);
		};
	}, []);

	return (
		<div>
			<h1>Minecraft Server Control Panel</h1>
			<div style={{ display: 'flex', alignItems: 'center' }}>
				<StatusIndicator status={status} />
				<button
					onClick={updateStatus}
					style={{ marginLeft: '10px', padding: '4px 8px', backgroundColor: '#007BFF', color: 'white', border: 'none', borderRadius: '4px', cursor: 'pointer' }}
				>
					🔄 Refresh
				</button>
			</div>
			<div style={{ margin: '20px 0' }}>
				<input
					type="text"
					placeholder="Path to server file (e.g. /path/to/run.bat)"
					value={filePath}
					onChange={e => setFilePath(e.target.value)}
					style={{ width: '60%', marginRight: '10px', padding: '8px' }}
				/>
				<button
					onClick={handleStart}
					disabled={isLoading || !status.toLowerCase().includes('not running')}
					style={{ marginRight: '10px', backgroundColor: !status.toLowerCase().includes('not running') ? '#ccc' : '#4CAF50', color: 'white', padding: '8px 16px', border: 'none', borderRadius: '4px', cursor: !status.toLowerCase().includes('not running') ? 'not-allowed' : 'pointer' }}
				>
					{isLoading ? 'Starting...' : 'Start Server'}
				</button>
				<button
					onClick={handleStop}
					disabled={isLoading || status.toLowerCase().includes('not running')}
					style={{ backgroundColor: status.toLowerCase().includes('not running') ? '#ccc' : '#f44336', color: 'white', padding: '8px 16px', border: 'none', borderRadius: '4px', cursor: status.toLowerCase().includes('not running') ? 'not-allowed' : 'pointer' }}
				>
					{isLoading ? 'Stopping...' : 'Stop Server'}
				</button>
			</div>
			{error && (
				<div style={{ color: 'red', margin: '10px 0' }}>
					{error}
				</div>
			)}
			{message && (
				<div style={{ margin: '10px 0' }}>
					{message}
				</div>
			)}
			<Console />
		</div>
	);
};

export default Dashboard;
