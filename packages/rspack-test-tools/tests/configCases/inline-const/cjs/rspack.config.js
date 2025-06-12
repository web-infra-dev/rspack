/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.cjs",
	optimization: {
		moduleIds: "named"
	},
	experiments: {
		inlineConst: true
	}
};
