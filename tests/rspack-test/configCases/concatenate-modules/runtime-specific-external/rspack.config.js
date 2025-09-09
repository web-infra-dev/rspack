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
				test: /value\.js/,
				sideEffects: false
			}
		]
	},
	optimization: {
		usedExports: true,
		innerGraph: true,
		sideEffects: true,
		concatenateModules: true,
		splitChunks: {
			cacheGroups: {
				shared: {
					test: /should-concat/,
					chunks: "all",
					minSize: 0,
					name: "shared"
				}
			}
		}
	}
};
