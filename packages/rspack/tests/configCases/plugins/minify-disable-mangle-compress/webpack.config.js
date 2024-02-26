const rspack = require("@rspack/core");
/**
 * @type {import("@rspack/core").Configuration}
 */
module.exports = {
	entry: {
		a: "./a",
		main: "./index"
	},
	output: {
		filename: "[name].js"
	},
	optimization: {
		minimize: true
	},
	plugins: [
		new rspack.SwcJsMinimizerRspackPlugin({
			mangle: false,
			compress: false
		})
	]
};
