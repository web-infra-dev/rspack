const rspack = require("@rspack/core");
module.exports = {
	mode: "production",
	output: {
		filename: "[name].js",
		chunkFormat: "module",
		chunkLoading: "import"
	},
	experiments: { outputModule: true },

	entry: {
		a: "./a.mjs",
		b: "./b.cjs",
		"c-cjs": "./c-cjs.js",
		"c-mjs": "./c-mjs.js",
		main: "./index"
	},

	optimization: {
		minimize: true,
		minimizer: [
			new rspack.SwcJsMinimizerRspackPlugin({
				exclude: [/(main|index)/]
			})
		]
	}
};
