/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		"a-name": {
			import: "./a",
			runtime: "a-runtime"
		},
		"b-name": {
			import: "./b",
			runtime: "b-runtime"
		},
		"ax-name": "./ax.js",
		"bx-name": "./bx.js"
	},
	target: "web",
	output: {
		filename: "[id].js"
	},
	node: false,
	optimization: {
		chunkIds: "named",
		moduleIds: "named",
		minimize: false,
		usedExports: true,
		concatenateModules: true,
		splitChunks: {
			cacheGroups: {
				forceMerge: {
					test: /shared/,
					enforce: true,
					name: "shared",
					chunks: "all"
				}
			}
		}
	},
	module: {
		rules: [
			{
				test: /dep/,
				sideEffects: false
			}
		]
	}
};
