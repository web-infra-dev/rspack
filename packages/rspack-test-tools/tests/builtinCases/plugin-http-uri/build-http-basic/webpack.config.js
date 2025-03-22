const { ServerPlugin } = require("./server.cjs");

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
			allowedUris: ["http://localhost:8999/", /^http:\/\/localhost:8999\/.*/],
			cacheLocation: "rspack-http-cache",
			lockfileLocation: "rspack-http-lockfile.json"
		}
	},
	plugins: [new ServerPlugin()]
};
