const { ProvideSharedPlugin } = require("@rspack/core").sharing;
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		filename: "[name].js"
	},
	optimization: {
		runtimeChunk: "single"
	},
	plugins: [
		new ProvideSharedPlugin({
			provides: ["x"]
		})
	]
};
