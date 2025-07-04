import { API_ENDPOINTS, DEFAULT_TIMEOUT, getApiUrl } from "./config.js";
// Shared API utilities
import { generateId } from "./nested-utils.js";

export const fetchWithTimeout = async (
	url,
	options = {},
	timeout = DEFAULT_TIMEOUT
) => {
	const controller = new AbortController();
	const timeoutId = setTimeout(() => controller.abort(), timeout);

	try {
		const response = await fetch(url, {
			...options,
			signal: controller.signal
		});
		clearTimeout(timeoutId);
		return response;
	} catch (error) {
		clearTimeout(timeoutId);
		throw error;
	}
};

export class ApiClient {
	constructor(baseUrl, headers = {}) {
		this.baseUrl = baseUrl;
		this.headers = headers;
		this.sessionId = generateId(); // Use imported function
	}

	async get(endpoint) {
		return fetchWithTimeout(`${this.baseUrl}${endpoint}`, {
			headers: this.headers
		});
	}

	async post(endpoint, data) {
		return fetchWithTimeout(`${this.baseUrl}${endpoint}`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
				...this.headers
			},
			body: JSON.stringify(data)
		});
	}
}

export const createApiClient = (baseUrl, headers) => {
	return new ApiClient(baseUrl, headers);
};

export default {
	fetchWithTimeout,
	ApiClient,
	createApiClient
};
