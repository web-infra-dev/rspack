/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		a: "./a",
		b: "./b"
	},
	output: {
		filename: "c-[id].js",
		libraryTarget: "commonjs2"
	},
	optimization: {
		chunkIds: "named",
		splitChunks: {
			cacheGroups: {
				shared: {
					chunks: "all",
					test: /shared/,
					filename: "shared-[id].js",
					enforce: true
				},
				common: {
					chunks: "all",
					filename: "common-[id].js",
					test: /common/,
					enforce: true
				},
				other: {
					chunks: "all",
					test: /other/,
					enforce: true
				}
			}
		}
	}
};
