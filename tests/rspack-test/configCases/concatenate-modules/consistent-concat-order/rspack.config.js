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
		minimize: false
	},
	experiments: {
		// inlineConst will inline [a-g].js into export-imported.js, so the order check will fail
		inlineConst: false,
	}
};
