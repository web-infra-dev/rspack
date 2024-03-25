/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index"
	},
	output: {
		filename: "[name].js"
	},
	optimization: {
		chunkIds: "named"
	},
	target: "node"
};
