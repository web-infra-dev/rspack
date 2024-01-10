const { rspack } = require("@rspack/core");

module.exports = {
	entry: {
		a: "./a",
		b: "./b",
		main: "./index"
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
