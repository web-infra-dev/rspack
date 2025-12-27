/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	cache: true,
	module: {
	},
	externals: {
		external: "var 123"
	}
};
