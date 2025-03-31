const supportsImportFn = require("../../../../dist/helper/legacy/supportsImportFn");
const path = require("path");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: path.join(__dirname, "a.js"),
				use: [
					{ loader: "./loader-2.js", parallel: true, options: {} },
					{
						loader: "builtin:test-no-passthrough-loader",
						parallel: true,
						options: {}
					},
					{ loader: "./loader-1.js", parallel: true, options: {} }
				]
			}
		]
	},
	experiments: {
		parallelLoader: supportsImportFn()
	}
};
