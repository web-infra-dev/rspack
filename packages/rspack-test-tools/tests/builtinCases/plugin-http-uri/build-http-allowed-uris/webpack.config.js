const { ServerPlugin } = require("./server.cjs");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	output: {
		libraryTarget: "commonjs2"
	},
	target: "node", // Use node target since we're using require
	externalsPresets: {
		web: false, // Disable web preset
		webAsync: false, // Disable webAsync preset
		node: false // Disable node preset
	},
	experiments: {
		buildHttp: {
			// Test both string and regex patterns for allowedUris
			allowedUris: [
				// Allow a specific path with a string (should allow allowed-module.js)
				"http://localhost:8999/allowed",

				// Allow paths matching a regex pattern (should match regex-module.js)
				/^http:\/\/localhost:8999\/regex.*/

				// Intentionally not including restricted-module.js to test blocking behavior
			],
			cacheLocation: "rspack-http-cache",
			lockfileLocation: "rspack-http-lockfile.json",
			http_client: require("./custom-http-client")
		}
	},
	plugins: [new ServerPlugin()]
};
