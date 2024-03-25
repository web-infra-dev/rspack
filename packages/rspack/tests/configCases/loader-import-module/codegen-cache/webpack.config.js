/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		publicPath: "/public/"
	},
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /app-proxy\.js/,
				loader: "./loader",
				options: {}
			}
		]
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	}
};
