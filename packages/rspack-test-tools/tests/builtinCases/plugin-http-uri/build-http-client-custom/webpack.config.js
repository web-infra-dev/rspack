/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	output: {
		libraryTarget: "commonjs2"
	},
	externalsPresets: {
		web: false, // Disable web preset
		webAsync: false, // Disable webAsync preset
		node: false // Disable node preset
	},
	experiments: {
		buildHttp: {
			allowedUris: ["http://localhost/", /^http:\/\/localhost\/.*/],
			cacheLocation: "rspack-http-cache",
			lockfileLocation: "rspack-http-lockfile.json",
			http_client: require("./custom-http-client")
		}
	},
	plugins: [
		{
			apply(compiler) {
				// Initialize plugin, used for testing
				compiler.hooks.beforeRun.tap("PreparePlugin", () => {
					// Clear the request tracking
					require("./custom-http-client").clearRequests();
				});
			}
		}
	]
};
