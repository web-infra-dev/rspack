module.exports = {
	optimization: {
		splitChunks: {
			cacheGroups: {
				async: {
					chunks: "async",
					test: /common/,
					minChunks: 1,
					reuseExistingChunk: false
					// enforce: true
				}
			}
		}
	}
};
