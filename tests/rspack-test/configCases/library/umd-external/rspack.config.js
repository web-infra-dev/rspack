/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: { type: "umd",
			root: "testLibrary[name]",
			amd: "test-library",
			commonjs: "test-library-[name]"
		}
	},
	externals: "module"
};
