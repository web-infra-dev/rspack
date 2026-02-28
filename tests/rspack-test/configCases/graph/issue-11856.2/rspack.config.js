/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		a: "./a",
		b: "./b"
	},
	target: "web",
	output: {
		filename: "[name].js",
		library: { type: "commonjs-module" }
	},
	optimization: {
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
		},
		// inlineExports will inline all shared-*.js, so there won't have a shared.js which is splitted out by splitChunks
		inlineExports: false
	},
};
