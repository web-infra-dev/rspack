/** @type {import("../../../../dist").Configuration} */
module.exports = {
	output: {
		libraryTarget: "umd",
		library: {
			root: "testLibrary[name]",
			amd: "test-library-[name]",
			commonjs: "test-library-[name]"
		},
		umdNamedDefine: true
	},
	externals: "module"
};
