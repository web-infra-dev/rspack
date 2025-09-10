/** @type {import("webpack").Configuration} */
module.exports = {
	entry: {
		a: "../runtime-condition/src/a",
		b: "../runtime-condition/src/b",
		c1: "../runtime-condition/src/c",
		c2: "../runtime-condition/src/c",
		ax: "../runtime-condition/src/ax",
		bx: "../runtime-condition/src/bx",
		cx1: "../runtime-condition/src/cx",
		cx2: "../runtime-condition/src/cx",
		d1: "../runtime-condition/src/d1",
		d2: "../runtime-condition/src/d2"
	},
	target: "web",
	mode: "production",
	devtool: false,
	output: {
		filename: "[name].js",
		library: { type: "commonjs-module" }
	},
	optimization: {
		minimize: false,
		moduleIds: "named",
		chunkIds: "named",
		providedExports: true,
		usedExports: true,
		concatenateModules: true,
		innerGraph: false,
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
	},
	experiments: {
		topLevelAwait: true
	}
};
