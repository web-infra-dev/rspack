const { rspack } = require("@rspack/core");

module.exports = {
	entry: {
		a: "./a",
		main: "./index"
	},
	optimization: {
		minimize: true,
		minimizer: [
			new rspack.SwcJsMinimizerRspackPlugin({
				comments: "some"
			})
		]
	}
};
