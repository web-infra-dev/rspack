const path = require("path");
const supportsImportFn = require("../../../../dist/helper/legacy/supportsImportFn");

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
					{ loader: "./loader-1.js", parallel: true, options: {} }
				]
			}
		]
	},
	experiments: {
		parallelLoader: supportsImportFn()
	}
};
