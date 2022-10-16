module.exports = {
	entry: {
		main: "./index.js"
	},
	optimization: {
		splitChunks: {
			cacheGroups: {
				vendor: {
					chunks: 'all',
					name: "vendor",
					test: 'foo'
				}
			}
		}
	}
};
