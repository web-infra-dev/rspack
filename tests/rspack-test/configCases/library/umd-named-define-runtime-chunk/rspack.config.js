/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: {
			import: "./index.js",
			runtime: "runtime"
		}
	},
	output: {
		filename: "[name].js",
		library: {
			type: "umd",
			root: "testLibrary[name]",
			amd: "test-library-[name]",
			commonjs: "test-library-[name]",
			umdNamedDefine: true
		},
	},
	externals: "module"
};
