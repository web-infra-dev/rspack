const path = require("path");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	extends: path.resolve(__dirname, "non-existent-config.js"),
	entry: "./src/index.js"
};
