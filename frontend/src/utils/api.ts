// Use absolute URL with port for the backend server
const BASE_URL = "http://localhost:8080";

async function handleResponse(response: Response): Promise<string> {
	if (!response.ok) {
		const errorText = await response.text();
		throw new Error(errorText || `HTTP error! status: ${response.status}`);
	}
	return response.text();
}

export async function startServer(): Promise<string> {
	try {
		const response = await fetch(`${BASE_URL}/start`, {
			method: "POST",
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
		console.log("Raw status response:", statusText);
		return statusText;
	} catch (error) {
		console.error("Status fetch error:", error);
		throw new Error(`Failed to fetch status: ${error instanceof Error ? error.message : 'Network error'}`);
	}
}
