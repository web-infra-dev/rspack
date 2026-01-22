/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: {
		entry: "./entry"
	},
	stats: {
		assets: true,
		modules: true,
	}
};
