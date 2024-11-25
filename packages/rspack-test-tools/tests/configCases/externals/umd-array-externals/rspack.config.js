/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		libraryTarget: "umd"
	},
	externals: {
		external: {
			root: ["a", "b"],
			commonjs: "a",
			commonjs2: "a",
			amd: "a"
		}
	}
};
