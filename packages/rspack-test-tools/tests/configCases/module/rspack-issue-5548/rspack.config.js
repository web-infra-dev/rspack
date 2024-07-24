/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	optimization: {
		minimize: false
	},
	module: {
		parser: {
			javascript: {
				dynamicImportMode: "eager"
			}
		}
	},
};
