const webpack = require('webpack')

module.exports = {
	stats: {
		all: false,
		modules: true,
	},
	plugins: [
		new webpack.IgnorePlugin({
			checkResource: (resource, request) => {
				if (resource.includes("zh") || resource.includes("globalIndex")) {
					return true;
				}
				return false;
			}
		})
	]
};
