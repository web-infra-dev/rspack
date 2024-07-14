const rspack = require("@rspack/core");
/**
 * @type {import("@rspack/core").Configuration}
 */
module.exports = {
	entry: {
		a: "./a.js",
		b: "./b.js",
		main: "./index.js"
	},
	output: {
		filename: "[name].js"
	},
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false
			}
		}
	},
	optimization: {
		minimize: true,
		minimizer: [
			new rspack.SwcCssMinimizerRspackPlugin({
				exclude: [/b\.css/]
			})
		]
	}
};
