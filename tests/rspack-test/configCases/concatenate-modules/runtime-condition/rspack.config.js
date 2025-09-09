/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	mode: "production",
	entry: {
		a: "./a.js",
		b: "./b.js"
	},
	output: {
		filename: "[name].js"
	},
	module: {
		rules: [
			{
				test: /js/,
				sideEffects: false
			}
		]
	},
	optimization: {
		concatenateModules: true,
		sideEffects: true,
		usedExports: true,
		innerGraph: true,
		splitChunks: {
			chunks: "all",
			minSize: 0,
			cacheGroups: {
				shared: {
					test: /(shared|utils|value)/,
					name: "shared",
					enforce: true
				}
			}
		}
	}
};
