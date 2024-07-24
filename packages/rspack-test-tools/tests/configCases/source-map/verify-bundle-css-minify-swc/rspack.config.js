const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false
			},
		},
	},
	devtool: "source-map",
	optimization: {
		minimize: true,
		minimizer: [
			new rspack.SwcCssMinimizerRspackPlugin()
		]
	},
	externals: ["source-map"],
	externalsType: "commonjs"
};
