const supportsImportFn = require("../../../../dist/helper/legacy/supportsImportFn");
/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /lib\.js/,
				use: [
					{
						loader: "./my-loader.js",
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
