const { SwcJsMinimizerRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: ["./index.js"],
		extract: ["./extract.js"],
		"no-extract": ["./no-extract.js"]
	},
	output: {
		filename: "[name].js"
	},
	plugins: [
		new SwcJsMinimizerRspackPlugin({
			extractComments: true,
			include: ["extract.js", "no-extract.js"]
		})
	]
};
