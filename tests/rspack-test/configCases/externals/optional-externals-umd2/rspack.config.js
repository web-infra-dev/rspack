/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: { type: "umd2" }
	},
	externals: {
		external: "external"
	}
};
