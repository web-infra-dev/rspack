module.exports = {
	output: {
		filename: "bundle.js",
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
					priority: 1,
					enforce: true
				},
				common: {
					chunks: "all",
					filename: "common-[name].js",
					priority: 2,
					test: /common/,
					enforce: true
				},
				other: {
					chunks: "all",
					test: /other/,
					filename: "other-[name].js",
					priority: 3,
					enforce: true
				}
			}
		}
	}
};
