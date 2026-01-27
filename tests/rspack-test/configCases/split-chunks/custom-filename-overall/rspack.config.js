/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	entry: {
		a: "./a",
		b: "./b"
	},
	output: {
		filename: "[name].js",
		library: { type: "commonjs2" }
	},
	optimization: {
		chunkIds: "named",
		splitChunks: {
			filename: "splitted-chunks/[name].js",
			cacheGroups: {
				shared: {
					chunks: "all",
					test: /shared/,
					filename: "shared-[name].js",
					enforce: true
				},
				common: {
					chunks: "all",
					test: /common/,
					enforce: true
				}
			}
		}
	}
};
