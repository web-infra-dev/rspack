/** @type {import("@rspack/core").Configuration[]} */
module.exports = {
	mode: "production",
	entry: "./index",
	optimization: {
		realContentHash: true
	},
	output: {
		filename: "[fullhash].js"
	}
};
