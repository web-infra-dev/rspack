const rspack = require("@rspack/core");
module.exports = [
	{
		entry: "./index.js",
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
			module: true
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
