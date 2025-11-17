module.exports = {
	entry: {
		main: "./index.js",
	},
	optimization: {
		sideEffects: true,
		splitChunks: {
			cacheGroups: {
				test: {
					test: /cjs\.js$/,
					name: "cjs-chunk",
				}
			}
		}
	}
}
