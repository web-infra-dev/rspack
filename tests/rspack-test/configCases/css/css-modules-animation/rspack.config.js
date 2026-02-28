const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	node: {
		__dirname: false,
		__filename: false
	},
	module: {
		generator: {
			"css/auto": {
				localIdentName: "[path][name]-[local]"
			}
		},
		rules: [
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	},
	optimization: {
		minimize: true,
		minimizer: [new rspack.LightningCssMinimizerRspackPlugin()],
		providedExports: true,
		usedExports: true
	},

};
