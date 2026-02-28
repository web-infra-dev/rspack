/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		a: {
			import: "./a.js",
			chunkLoading: "async-node"
		},
		b: {
			import: "./b.js",
			chunkLoading: "require"
		}
	},
	output: {
		filename: "[name].js"
	}
};
