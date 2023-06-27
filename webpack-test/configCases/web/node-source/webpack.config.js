/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	entry: "./index.mjs",
	performance: {
		hints: false
	},
	optimization: {
		minimize: false
	}
};
