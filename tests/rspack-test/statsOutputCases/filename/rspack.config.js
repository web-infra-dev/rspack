const path = require("path");

/** @type {import('@rspack/core').Configuration} */
module.exports = {
	entry: "./index",
	output: {
		filename: "[id].xxxx.js",
		chunkFilename: "[id].xxxx.js"
	},
	stats: {
		assets: true,
		modules: true,
	}
};
