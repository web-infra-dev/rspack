module.exports = {
	optimization: {
		splitChunks: {
			cacheGroups: {
				ab: {
					test: /[ab]\.js$/,
					name: "ab-chunk",
					chunks: "all",
				}
			}
		}
	}
}
