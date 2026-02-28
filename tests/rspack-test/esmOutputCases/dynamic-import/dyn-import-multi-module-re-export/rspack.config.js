module.exports = {
	externals: {
		'fs': 'module fs',
		'path': 'module path'
	},
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
