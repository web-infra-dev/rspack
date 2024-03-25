const webpack = require('webpack')

/** @type {import('@rspack/core').Configuration} */
module.exports = {
	entry: "./index",
	stats: {
		all: false,
		modules: true,
	},
	plugins: [
		new webpack.IgnorePlugin({
			checkResource: (resource, request) => {
				if (resource.includes("zh") || resource.includes("globalIndex")) {
					return true;
				}
				return false;
			}
		})
	]
};
