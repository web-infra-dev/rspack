/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		module: true,
		filename: "[name].mjs",
		library: { type: "module" }
	},
	optimization: {
		runtimeChunk: "single" // any value other than `false`
	}
};
