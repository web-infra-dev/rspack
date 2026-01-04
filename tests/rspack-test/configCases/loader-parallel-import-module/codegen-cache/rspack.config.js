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
				use: [
					{
						loader: "./loader",
						options: {},
						parallel: { maxWorkers: 4 }
					}
				]
			}
		]
	},
	experiments: {
		parallelLoader: true
	}
};
