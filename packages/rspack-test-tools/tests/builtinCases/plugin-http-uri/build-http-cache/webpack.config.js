const ServerPlugin = require("./server");
const path = require("path");
const fs = require("fs");

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
			cacheLocation: path.resolve(__dirname, "rspack-http-cache"),
			lockfileLocation: path.resolve(__dirname, "rspack-http-lockfile.json")
		}
	},
	plugins: [
		new ServerPlugin(),
		{
			apply(compiler) {
				// Clean up cache files before each run
				compiler.hooks.beforeRun.tapPromise("CleanupPlugin", async () => {
					try {
						if (fs.existsSync(path.resolve(__dirname, "rspack-http-cache"))) {
							fs.rmSync(path.resolve(__dirname, "rspack-http-cache"), {
								recursive: true,
								force: true
							});
						}
						if (
							fs.existsSync(
								path.resolve(__dirname, "rspack-http-lockfile.json")
							)
						) {
							fs.unlinkSync(
								path.resolve(__dirname, "rspack-http-lockfile.json")
							);
						}
					} catch (e) {
						console.error("Error cleaning up cache files:", e);
					}
				});
			}
		}
	]
};
