/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",

	entry: {
		main: "./index"
	},
	target: "node",
	output: {
		filename: "[name].js"
	},
	optimization: {
		chunkIds: "named",
		moduleIds: "named"
	}
};
