/** @type {import("webpack").Configuration} */
module.exports = {
	entry: {
		a: "./src/a",
		b: "./src/b",
		c1: "./src/c",
		c2: "./src/c",
		ax: "./src/ax",
		bx: "./src/bx",
		cx1: "./src/cx",
		cx2: "./src/cx",
		d1: "./src/d1",
		d2: "./src/d2"
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
		concatenateModules: false,
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
