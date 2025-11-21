/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./index",
	performance: false,
	stats: {
		modulesSpace: 20
	}
};
