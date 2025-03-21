// Setup custom HTTP client for testing
console.log("Initializing custom HTTP client...");

// Track HTTP requests
window.requests = [];
window.getRequestCount = () => {
	return window.requests.length;
};

// Create the custom HTTP client function matching the expected interface
window.customHttpClient = (url, headers) => {
	console.log("🔥 Custom client fetching URL:", url);

	// Track this request
	window.requests.push({ url, headers, timestamp: new Date() });

	// Return a promise that resolves to the response
	return fetch(url, { headers }).then(response => {
		// Convert the response to the format expected by the HTTP client
		return response.arrayBuffer().then(buffer => {
			// Extract headers
			const responseHeaders = {};
			response.headers.forEach((value, key) => {
				responseHeaders[key] = value;
			});

			console.log(
				`✅ Custom client fetched ${url} successfully (${buffer.byteLength} bytes)`
			);

			// Return the standardized format
			return {
				status: response.status,
				headers: responseHeaders,
				body: new Uint8Array(buffer)
			};
		});
	});
};

console.log("Custom HTTP client initialized");
