const { rspack } = require("@rspack/core");

module.exports = {
	optimization: {
		minimize: true,
		minimizer: [
			new rspack.SwcJsMinimizerRspackPlugin({
				keepClassNames: true
			})
		]
	}
};
