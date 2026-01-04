/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /\.png$/,
				use: [{ loader: "./loader.js", parallel: { maxWorkers: 4 }, options: {} }],
				type: "asset/resource"
			}
		]
	},
	experiments: {
		parallelLoader: true
	}
};
