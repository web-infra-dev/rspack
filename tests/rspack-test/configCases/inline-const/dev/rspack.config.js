/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	output: {
		pathinfo: false
	},
	optimization: {
		inlineExports: true
	}
};
