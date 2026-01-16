const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: ["web", "browserslist:safari >= 4"],
	node: {
		__dirname: false,
		__filename: false,
	},
	optimization: {
		minimize: true,
		minimizer: [
			new rspack.SwcJsMinimizerRspackPlugin({
				minimizerOptions: {
					ecma: 2020,
				}
			}),
			new rspack.LightningCssMinimizerRspackPlugin({
				minimizerOptions: {
					targets: "chrome > 95",
				}
			})
		]
	},
};
