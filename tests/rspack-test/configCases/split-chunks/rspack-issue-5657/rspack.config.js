/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	entry: {
		main: "./src/index.js"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				type: 'css/auto'
			}
		]
	},
	optimization: {
		splitChunks: {
			chunks: "all",
			cacheGroups: {
				lib: {
					test: /[\/\\]src\/lib[\/\\]/,
					minSize: 0,
					maxSize: 50,
					minChunks: 1
				}
			}
		}
	}
};
