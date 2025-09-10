const { rspack } = require("@rspack/core");
module.exports = {
	entry: "./index.js",
	target: "web",
	devtool: false,
	optimization: {
		minimize: true,
		minimizer: [new rspack.SwcJsMinimizerRspackPlugin()]
	},
	module: {
		noParse: /index\.js$/
	}
};
