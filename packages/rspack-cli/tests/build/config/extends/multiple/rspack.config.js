const path = require("path");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	extends: [
		path.resolve(__dirname, "base.rspack.config.js"),
		path.resolve(__dirname, "dev.rspack.config.js")
	],
	entry: "./src/index.js",
	output: {
		path: path.resolve(__dirname, "dist")
	}
};
