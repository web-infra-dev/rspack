const rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		generator: {
			"css/auto": {
				localIdentName: "[path][name]-[local]",
				exportsOnly: false,
			}
		}
	},
	optimization: {
		minimize: true,
		minimizer: [
			new rspack.LightningCssMinimizerRspackPlugin(),
		]
	},
	experiments: {
		css: true
	}
};
