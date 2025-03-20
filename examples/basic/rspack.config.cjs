module.exports = {
	context: __dirname,
	mode: "development",
	devtool: false,
	entry: {
		main: './index.js'
	},
	// Disable external presets to prevent HTTP URIs from being automatically treated as externals
	externalsPresets: {
		web: false,
		webAsync: false
	},
	experiments: {
		buildHttp: {
			allowedUris: ["https://esm.sh/.*"],
			// Minimal HTTP client implementation using fetch
			http_client: (url, headers) => {
				return fetch(url, { headers }).then(res => 
					res.arrayBuffer().then(buffer => ({
						status: res.status,
						headers: Object.fromEntries(res.headers.entries()),
						body: Buffer.from(buffer)
					}))
				);
			}
		}
	}
}
