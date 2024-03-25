const path = require("path");

/** @type {import('@rspack/core').Configuration} */
module.exports = {
	entry: "./index",
	output: {
		path: path.join(__dirname, './dist'),
		filename: "[id].xxxx.js",
		chunkFilename: "[id].xxxx.js"
	}
};
