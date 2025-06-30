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
				format: {
					preamble: "function f() {}\n"
				}
			}
		})
	]
};
