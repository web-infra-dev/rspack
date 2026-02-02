/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		module: true,
		filename: "[name].mjs"
	},
	target: ["web", "es2020"],
	experiments: {
		},
	optimization: {
		minimize: true,
		runtimeChunk: "single"
	}
};
