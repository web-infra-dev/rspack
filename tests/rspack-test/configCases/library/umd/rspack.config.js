/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: {
			type: "umd",
			root: "testLibrary",
			amd: "test-library",
			commonjs: "test-library"
		}
	}
};
