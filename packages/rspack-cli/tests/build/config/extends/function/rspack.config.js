const path = require("path");

/**
 * @type {function(any, any): import('@rspack/core').RspackOptions}
 */
module.exports = (env, argv) => {
	return {
		extends: path.resolve(__dirname, "base.rspack.config.js"),
		entry: "./src/index.js",
		output: {
			path: path.resolve(__dirname, "dist")
		}
	};
};
