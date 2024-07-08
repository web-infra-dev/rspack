const rspack = require("@rspack/core");
module.exports = [
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
	},
	{
		entry: {
			main: "./module-entry.js",
			module: "./module.js"
		},
		output: {
			module: true,
			filename: "[name].mjs"
		},
		experiments: {
			outputModule: true
		},
		target: "es2022",
		externalsPresets: { web: true },
		optimization: {
			minimize: true,
			minimizer: [new rspack.SwcJsMinimizerRspackPlugin()]
		}
	}
];
