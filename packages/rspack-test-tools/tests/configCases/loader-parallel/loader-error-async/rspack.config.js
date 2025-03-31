const supportsImportFn = require("../../../../dist/helper/legacy/supportsImportFn");
const path = require("path");
const file = path.resolve(__dirname, "lib.js");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: file,
				resourceQuery: /async/,
				use: [
					{
						loader: "./async.js",
						parallel: true,
						options: {}
					}
				]
			},
			{
				test: file,
				resourceQuery: /callback/,
				use: [
					{
						loader: "./callback.js",
						parallel: true,
						options: {}
					}
				]
			}
		]
	},
	experiments: {
		parallelLoader: supportsImportFn()
	}
};
