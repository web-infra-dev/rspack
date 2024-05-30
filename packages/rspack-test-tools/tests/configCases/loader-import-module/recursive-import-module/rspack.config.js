/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		publicPath: "/public/"
	},
	entry: "./entry.js",
	module: {
		rules: [
			{
				test: /\.js/,
				loader: './loader.js'
			}
		]
	},
};
