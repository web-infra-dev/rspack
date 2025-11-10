module.exports = {
	optimization: {
		runtimeChunk: false,
		splitChunks: {
			cacheGroups: {
				module: {
					test: /module\.js$/,
				},
			}
		}
	}
}
