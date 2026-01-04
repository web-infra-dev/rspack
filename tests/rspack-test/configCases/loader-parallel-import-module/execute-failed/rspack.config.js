/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /index\.js/,
				use: [
					{ loader: "./import-loader.js", options: {}, parallel: { maxWorkers: 4 } },
					{ loader: "./import-loader-2.js", options: {}, parallel: { maxWorkers: 4 } }
				]
			}
		]
	},
	experiments: {
		parallelLoader: true
	}
};
