/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	resolve: {
		byDependency: {
			esm: {
				extensions: [".bar", "..."]
			}
		}
	}
};
