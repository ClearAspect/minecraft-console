// Use dynamic URL that works on any machine - frontend and backend run on same host
const BASE_URL = `http://${window.location.hostname}:8080`;

async function handleResponse(response: Response): Promise<string> {
	if (!response.ok) {
		const errorText = await response.text();
		throw new Error(errorText || `HTTP error! status: ${response.status}`);
	}
	return response.text();
}

export async function startServer(filePath: string): Promise<string> {
	try {
		const response = await fetch(`${BASE_URL}/start`, {
			method: "POST",
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ file_path: filePath }),
		});
		return handleResponse(response);
	} catch (error) {
		throw new Error(`Failed to start server: ${error instanceof Error ? error.message : 'Unknown error'}`);
	}
}

export async function stopServer(): Promise<string> {
	try {
		const response = await fetch(`${BASE_URL}/stop`, {
			method: "POST",
		});
		return handleResponse(response);
	} catch (error) {
		throw new Error(`Failed to stop server: ${error instanceof Error ? error.message : 'Unknown error'}`);
	}
}

export async function fetchStatus(): Promise<string> {
	try {
		const response = await fetch(`${BASE_URL}/status`);
		const statusText = await handleResponse(response);
		return statusText;
	} catch (error) {
		throw new Error(`Failed to fetch status: ${error instanceof Error ? error.message : 'Network error'}`);
	}
}
