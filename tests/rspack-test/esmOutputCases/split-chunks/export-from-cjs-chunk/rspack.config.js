module.exports = {
	entry: {
		main: "./index.js",
	},
	optimization: {
		sideEffects: true,
		splitChunks: {
			chunks: "all",
			minSize: 0,
			cacheGroups: {
				test: {
					test: /cjs\.js$/,
					name: "cjs-chunk",
				}
			}
		}
	}
}
