const rspack = require("@rspack/core");
/**
 * @type {import("@rspack/core").Configuration}
 */
module.exports = {
	entry: {
		a: "./a",
		b: "./b",
		main: "./index"
	},
	output: {
		filename: "[name].js"
	},
	optimization: {
		minimize: true,
		minimizer: [
			new rspack.SwcJsMinimizerRspackPlugin({
				exclude: [/b\.js/]
			})
		]
	}
};
