const ServerPlugin = require("./server");
const httpClient = require("./custom-http-client");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	output: {
		libraryTarget: "commonjs2"
	},
	target: "node",
	externalsPresets: {
		web: false, // Disable web preset
		webAsync: false, // Disable webAsync preset
		node: false // Disable node preset
	},
	experiments: {
		buildHttp: {
			// No allowedUris specified - testing default behavior
			// The bug report states that when no conditions are specified,
			// the behavior should be to allow all URIs by default
			cacheLocation: "rspack-http-cache-default",
			lockfileLocation: "rspack-http-lockfile-default.json",
			http_client: httpClient
		}
	},
	plugins: [new ServerPlugin()]
};
