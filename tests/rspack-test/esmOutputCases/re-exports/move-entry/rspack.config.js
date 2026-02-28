module.exports = {
	entry: {
		main: "./index.js",
		lib: "./lib.js",
	},
	optimization: {
		splitChunks: {
			cacheGroups: {
				main: {
					test: /index\.js/,
					name: 'proxy-main',
					enforce: true,
				}
			}
		}
	}
}
