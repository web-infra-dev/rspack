const path = require("path");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	devtool: "source-map",
	module: {
		rules: [
			{
				test: path.join(__dirname, "logo.png"),
				use: [{ loader: "./empty-loader.js", parallel: true, options: {} }],
				type: "asset/resource"
			}
		]
	},
	experiments: {
		parallelLoader: {
			maxWorkers: 8,
		}
	}
};
