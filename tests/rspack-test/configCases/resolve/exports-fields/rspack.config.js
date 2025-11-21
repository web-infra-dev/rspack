/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	resolve: {
		exportsFields: ["a", "b"]
	}
};
