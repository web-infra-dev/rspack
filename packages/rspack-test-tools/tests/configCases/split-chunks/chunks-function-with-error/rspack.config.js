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
				splitLib2: {
					chunks(chunk) {
						throw new Error("CHUNKS_FUNCTION_WITH_ERROR")
					},
					test: /\.js/,
				}
			}
		}
	}
};
