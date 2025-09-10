/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		bundle: "./src/index.js",
		another: "./src/another.js"
	},
	output: {
		filename: "[name].js"
	},
	optimization: {
		splitChunks: {
			minSize: 10,
			cacheGroups: {
				lodash: {
					test: /lodash/,
					name: "lodash",
					chunks: "all",
					priority: 3
				},
				terser: {
					test: /terser/,
					name: "terser",
					chunks: "all",
					priority: 2
				},
				acorn: {
					test: /acorn/,
					name: "acorn",
					chunks: "all",
					priority: 1
				}
			}
		}
	},
	target: "node"
};
