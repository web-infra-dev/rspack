/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		publicPath: "/public/"
	},
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /a.js/,
				loader: './convert-loader.js'
			},
		]
	},
};
