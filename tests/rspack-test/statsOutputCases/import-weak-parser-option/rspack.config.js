/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: {
		entry: "./entry"
	},
	module: {
		parser: {
			javascript: {
				dynamicImportMode: "weak"
			}
		}
	}
};
