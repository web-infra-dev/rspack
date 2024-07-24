/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		filename: "[name].js"
	},
	target: "web",
	optimization: {
		runtimeChunk: true
	}
};
