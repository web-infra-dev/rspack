var webpack = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	node: {
		__dirname: false,
		__filename: false
	},
	entry: "./index.js",
	output: {
		filename: "[name].js"
	},
	plugins: [new webpack.optimize.LimitChunkCountPlugin({ maxChunks: 1 })]
};
