/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	entry: {
		a: "./a",
		b: "./b"
	},
	output: {
		filename: "[name].js",
		libraryTarget: "commonjs2"
	},
	optimization: {
		chunkIds: "named",
		splitChunks: {
			cacheGroups: {
				shared: {
					chunks: "all",
					test: /shared/,
					filename: "shared-[name].js",
					reuseExistingChunk: false,
					enforce: true
				},
				common: {
					chunks: "all",
					test: /common/,
					reuseExistingChunk: false,
					enforce: true
				}
			}
		}
	}
};
