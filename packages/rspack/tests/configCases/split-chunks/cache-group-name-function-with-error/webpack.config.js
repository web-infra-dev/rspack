/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index"
	},
	output: {
		filename: "[name].js"
	},
	target: "async-node",
	optimization: {
		splitChunks: {
			chunks: "all",
			minSize: 0,
			cacheGroups: {
				foo: {
					test: /\.js/,
					name(module, chunks) {
						throw new Error("CACHE_GROUP_NAME_FUNCTION_WITH_ERROR")
					}
				}
			}
		}
	}
};
