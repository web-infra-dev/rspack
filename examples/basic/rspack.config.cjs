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
			http_client: (url, headers) => {
				console.log('URL', url, 'HEADERS', headers);
				return fetch(url, { headers }).then(res => {
					return res.arrayBuffer().then(buffer => {
						console.log(buffer);
						return {
							status: res.status,
							headers: Object.fromEntries(res.headers.entries()),
							body: Buffer.from(buffer)
						};
					});
				});
			}
		}
	}
}
