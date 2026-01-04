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
						parallel: { maxWorkers: 4 }
					}
				],
				issuerLayer: "main"
			}
		]
	},
	experiments: {
		parallelLoader: true
	}
};
