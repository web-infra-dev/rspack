/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: { type: "umd",
			root: ["test", "library"],
			amd: "test-library",
			commonjs: "test-library"
		}
	}
};
