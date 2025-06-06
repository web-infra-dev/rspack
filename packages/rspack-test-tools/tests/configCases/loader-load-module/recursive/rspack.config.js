/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		publicPath: "/public/"
	},
	entry: "./entry.js",
	module: {
		rules: [
			{
				test: /lib\.js/,
				loader: "./loader.js"
			}
		]
	}
};
