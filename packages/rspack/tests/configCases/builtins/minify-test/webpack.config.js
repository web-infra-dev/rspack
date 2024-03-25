const { rspack } = require("@rspack/core");

module.exports = {
	entry: {
		a: "./a",
		a2: "./a2",
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
				test: [/a\d?\.js/],
				exclude: [/a\.js/]
			})
		]
	}
};
