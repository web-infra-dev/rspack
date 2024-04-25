/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index"
	},
	mode: "development",
	target: "node",
	output: {
		filename: "[name].js"
	},
	optimization: {
		chunkIds: "named",
		splitChunks: {
			cacheGroups: {
				js: {
					chunks: "all",
					test: /(foo|async)/,
					enforce: true,
					name: "js-chunk"
				}
			}
		}
	}
};
