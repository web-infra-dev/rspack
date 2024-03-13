module.exports = {
	entry: {
		main: "./index1.js",
		main2: "./index2.js",
		main3: "./index3.js"
	},
	output: {
		chunkFilename: "[id].[contenthash].js",
		filename: '[name].js'
	},
	optimization: {
		splitChunks: {
			cacheGroups: {
				common: {
					chunks: "all",
					test: /common/,
					enforce: true,
					name: "common"
				},
				share: {
					chunks: "all",
					test: /share/,
					enforce: true,
					name: "share"
				}
			}
		},
		runtimeChunk: "single",
		chunkIds: 'named'
	}
};
