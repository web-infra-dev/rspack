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
				mangle: {
					keep_classnames: true
				}
			}
		})
	]
};
