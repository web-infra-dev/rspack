/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "production",
	entry: "./index",
	performance: false,
	stats: {
		assets: true,
		modules: true,
		modulesSpace: Infinity,
		modulesSort: "!name"
	}
};
