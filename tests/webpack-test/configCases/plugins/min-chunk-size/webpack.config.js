var webpack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new webpack.optimize.MinChunkSizePlugin({
			minChunkSize: 30
		})
	]
};
