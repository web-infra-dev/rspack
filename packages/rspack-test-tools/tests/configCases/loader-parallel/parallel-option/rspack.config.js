const supportsImportFn = require("../../../../dist/helper/legacy/supportsImportFn");
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
	experiments: {
		parallelLoader: supportsImportFn()
	}
};
