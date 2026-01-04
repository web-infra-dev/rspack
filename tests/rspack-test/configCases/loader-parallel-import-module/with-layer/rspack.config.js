/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.js/,
				use: [
					{
						loader: "./loader",
						options: {},
						parallel: true
					}
				],
				issuerLayer: "main"
			}
		]
	},
	experiments: {
		parallelLoader: {
			maxWorkers: 8,
		}
	}
};
