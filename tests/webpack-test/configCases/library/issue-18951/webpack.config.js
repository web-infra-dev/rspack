/** @type {import("@rspack/core").Configuration} */
module.exports = {
	experiments: { outputModule: true },
	output: {
		filename: "[name].mjs",
		library: { type: "module" }
	},
	optimization: {
		runtimeChunk: "single" // any value other than `false`
	}
};
