const path = require("path");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	extends: path.resolve(__dirname, "core.rspack.config.js"),
	output: {
		filename: "base.bundle.js"
	}
};
