const rspack = require("@rspack/core");
const { supportsImportFn } = require("@rspack/test-tools");

module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /lib\.js/,
				use: [
					{
						loader: "./loader-in-worker.js",
						parallel: true,
						options: {}
					}
				]
			}
		]
	},
	plugins: [
		new rspack.DefinePlugin({
			SUPPORTS_IMPORT_FN: supportsImportFn()
		})
	],
	experiments: {
		parallelLoader: supportsImportFn()
	}
};
