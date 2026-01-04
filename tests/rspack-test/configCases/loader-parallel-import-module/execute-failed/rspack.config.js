/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /index\.js/,
				use: [
					{ loader: "./import-loader.js", options: {}, parallel: true },
					{ loader: "./import-loader-2.js", options: {}, parallel: true }
				]
			}
		]
	},
	experiments: {
		parallelLoader: {
			maxWorkers: 8,
		}
	}
};
