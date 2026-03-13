module.exports = {
	optimization: {
		splitChunks: {
			cacheGroups: {
				middle: {
					test: /middle\.js$/,
					name: "middle-chunk",
				},
				leaf: {
					test: /leaf\.js$/,
					name: "leaf-chunk",
				}
			}
		}
	}
}
