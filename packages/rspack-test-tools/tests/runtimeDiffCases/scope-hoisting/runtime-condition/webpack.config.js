/** @type {import("webpack").Configuration} */
module.exports = {
	mode: "production",
	entry: {
		a: "./a.mjs",
		b: "./b.mjs"
	},
	output: {
		filename: "[name].js"
	},
	module: {
		rules: [
			{
				test: /mjs/,
				sideEffects: false
			},
			{
				test: /cjs/,
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
