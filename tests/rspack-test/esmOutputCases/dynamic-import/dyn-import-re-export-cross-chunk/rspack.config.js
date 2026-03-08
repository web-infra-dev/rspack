module.exports = {
	optimization: {
		splitChunks: {
			cacheGroups: {
				shared: {
					test: /shared\.js$/,
					name: "shared-chunk",
				}
			}
		}
	}
}
