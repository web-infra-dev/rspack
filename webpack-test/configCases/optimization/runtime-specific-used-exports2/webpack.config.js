/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		filename: "[name].js"
	},
	target: "node",
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	},
	optimization: {
		chunkIds: "named",
		usedExports: true,
		splitChunks: {
			chunks: "all",
			cacheGroups: {
				defaultVendors: {
					test: /[\\/]node_modules[\\/]/,
					enforce: true
				}
			}
		}
	},
	entry: {
		a: "./1",
		b: "./2",
		c: "./3"
	}
};
