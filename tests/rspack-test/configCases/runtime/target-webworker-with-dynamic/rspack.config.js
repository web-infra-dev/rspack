/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "webworker",
	devtool: false,
	output: {
		filename: "[name].js"
	},
	optimization: {
		runtimeChunk: "single"
	}
};
