// Custom HTTP client implementation that handles both static and dynamic imports
const fetch = global.fetch || require("node-fetch");

// Track for tests
const requests = [];

/**
 * Custom HTTP client function
 * This client includes support for dynamic import callbacks that are needed for tests
 * @param {string} url - The URL to fetch
 * @param {Record<string, string>} headers - Request headers
 */
function customHttpClient(url, headers) {
	// Track the request for tests
	requests.push({ url, headers });

	console.log("\n🔥 Fetching URL:", url);

	// Return a promise that resolves to the response
	return (
		fetch(url, { headers })
			.then(response => {
				// Convert the response to the format expected by the HTTP client
				return response.arrayBuffer().then(buffer => {
					// Extract headers
					const responseHeaders = {};
					response.headers.forEach((value, key) => {
						responseHeaders[key] = value;
					});

					console.log(
						`✅ Fetched ${url} successfully (${buffer.byteLength} bytes)`
					);

					// Return the standardized format
					return {
						status: response.status,
						headers: responseHeaders,
						body: Buffer.from(buffer)
					};
				});
			})
			// Handle errors - crucial for dynamic imports
			.catch(error => {
				console.error(`❌ Error fetching ${url}:`, error);
				throw error;
			})
	);
}

// For testing purposes
customHttpClient.getRequests = () => [...requests];
customHttpClient.clearRequests = () => {
	requests.length = 0;
};

// Export both the client and utility functions
module.exports = customHttpClient;
