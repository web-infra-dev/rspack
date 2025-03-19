const { rspack } = require("@rspack/core");
const path = require("path");

/** @type {function(any, any): import("@rspack/core").Configuration[]} */
module.exports = (env, { testPath }) => {
	return {
		entry: {
			consume: "./consume.js",
			main: "./index.js"
		},
		resolve: {
			alias: {
				library: path.resolve(
					testPath,
					"../0-concatenation-tree-shaking/main.mjs"
				)
			}
		},
		output: {
			module: true,
			chunkFormat: "module",
			filename: "[name].mjs"
		},
		experiments: {
			outputModule: true
		},
		optimization: {
			minimize: true,
			concatenateModules: true
		},
		plugins: [
			new rspack.SwcJsMinimizerRspackPlugin({
				minimizerOptions: {
					minify: false,
					mangle: false
				}
			})
		]
	};
};
