module.exports = {
	context: __dirname,
	mode: 'development',
	devtool: false,
	entry: {
		main: './index.js'
	},
	externals: [],  // Don't treat any modules as external
	externalsPresets: {
		web: false,  // Disable web preset
		webAsync: false, // Disable webAsync preset
		node: false  // Disable node preset
	},
	experiments: {
		buildHttp: {
			allowedUris: [
				/^https?:\/\/.*/  // Regex literal that matches HTTP and HTTPS URLs
			],
			cacheLocation: './rspack-http-cache',
			lockfileLocation: './rspack-http-lockfile.json',
			// Custom HTTP client implementation
			http_client: (url, headers) => {
				console.log('\n🔥 Custom client fetching URL:', url);
				
				// Return a promise that resolves to the response
				return fetch(url, { headers })
					.then(response => {
						// Convert the response to the format expected by the HTTP client
						return response.arrayBuffer().then(buffer => {
							// Extract headers
							const responseHeaders = {};
							response.headers.forEach((value, key) => {
								responseHeaders[key] = value;
							});
							
							console.log(`✅ Custom client fetched ${url} successfully (${buffer.byteLength} bytes)`);
							
							// Return the standardized format
							return {
								status: response.status,
								headers: responseHeaders,
								body: Buffer.from(buffer)
							};
						});
					});
			}
		}
	}
}
