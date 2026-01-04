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
				use: [
					{
						loader: "./convert-loader.js",
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
