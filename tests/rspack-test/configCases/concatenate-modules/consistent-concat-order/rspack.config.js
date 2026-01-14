/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.js/,
				sideEffects: false
			}
		]
	},
	output: {
		filename: "bundle.js"
	},
	optimization: {
		concatenateModules: true,
		sideEffects: true,
		moduleIds: "named",
		minimize: false,
		// inlineExports will inline all shared-*.js, so there won't have a shared.js which is splitted out by splitChunks
		inlineExports: false
	},
};
