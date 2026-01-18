/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	mode: "development",
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	},
	optimization: {
		splitChunks: {
			cacheGroups: {
				css: {
					type: "css/auto",
					enforce: true,
					name: "css"
				}
			}
		}
	},
	externalsPresets: {
		node: true
	},
	node: {
		__dirname: false
	}
};
