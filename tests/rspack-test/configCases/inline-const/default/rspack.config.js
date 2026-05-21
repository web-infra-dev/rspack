/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	output: {
		filename: "bundle.js"
	},
	optimization: {
		moduleIds: "named",
		inlineExports: true
	}
};
