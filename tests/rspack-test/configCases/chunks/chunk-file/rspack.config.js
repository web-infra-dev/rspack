/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "node",
	entry: {
		main: {
			import: "./index.js",
			filename: "main"
		}
	},
	optimization: {
		splitChunks: {
			cacheGroups: {
				vendor: {
					chunks: "all",
					reuseExistingChunk: true,
					test: /[\\/]node_modules[\\/]/,
					minSize: 0,
					minChunks: 1
				}
			}
		}
	}
};
