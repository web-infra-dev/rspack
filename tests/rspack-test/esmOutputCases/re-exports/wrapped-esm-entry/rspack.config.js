module.exports = {
	externals: [
		{fs: "module fs"}
	],
	optimization: {
		splitChunks: {
			cacheGroups: {
				main: {
					test: /index\.js/,
					name: 'main-chunk',
					chunks: 'all',
					enforce: true
				}
			}
		}
	}
}
