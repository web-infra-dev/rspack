/** @type {import("@rspack/test-tools").TBuiltinCaseConfig} */
module.exports = {
	description: "should collect JSON module sizes with tree-shaking applied",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./index.js"
		};
	}
};
