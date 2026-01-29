/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: { type: "commonjs2" }
	},
	externals: {
		external: "external"
	}
};
