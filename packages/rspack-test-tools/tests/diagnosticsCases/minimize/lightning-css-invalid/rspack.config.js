const rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	target: "web",
	optimization: {
		minimize: true,
		minimizer: [
			new rspack.LightningCssMinimizerRspackPlugin(),
		]
	},
	experiments: {
		css: true,
	}
};
