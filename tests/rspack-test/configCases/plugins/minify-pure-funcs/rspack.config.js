const { rspack } = require("@rspack/core");
/**
 * @type {import("@rspack/core").Configuration}
 */
module.exports = {
	optimization: {
		minimize: true
	},
	plugins: [
		new rspack.SwcJsMinimizerRspackPlugin({
			minimizerOptions: {
				compress: {
					pure_funcs: ["__logger.error", "__logger.warn"]
				}
			}
		})
	]
};
