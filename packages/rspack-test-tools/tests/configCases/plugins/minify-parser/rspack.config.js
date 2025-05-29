const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = [
	{
		entry: {
			importAttributes: "./importAttributes.mjs"
		},
		output: {
			filename: "[name].js",
			module: true
		},
		externalsType: "module",
		externals: ["./a.json"],
		experiments: {
			outputModule: true
		},
		optimization: {
			minimize: true,
			minimizer: [new rspack.SwcJsMinimizerRspackPlugin()]
		}
	},
	{
		entry: {
			main: "./index.js"
		},
		output: {
			filename: "[name].js"
		},
		externalsPresets: {
			node: true
		}
	}
];
