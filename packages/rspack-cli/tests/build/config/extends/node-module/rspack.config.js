const path = require("path");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	extends: "mock-rspack-config",
	entry: "./src/index.js",
	output: {
		path: path.resolve(__dirname, "dist")
	}
};
