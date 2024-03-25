/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	entry: {
		a: "./a",
		b: "./b"
	},
	output: {
		filename: "c-[name].js",
		libraryTarget: "commonjs2",
		// TODO:
		// here is a webpack bug and also in rspack
		// if do not set chunkFilename and hash
		// __webpack_require__.h will not exists
		chunkFilename: "[hash].js"
	},
	optimization: {
		chunkIds: "named",
		splitChunks: {
			cacheGroups: {
				shared: {
					chunks: "all",
					test: /shared/,
					filename: "shared-[name]-[hash:6].js",
					enforce: true
				},
				common: {
					chunks: "all",
					filename: "common-[name]-[fullhash:4].js",
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
