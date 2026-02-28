/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		library: { type: "umd", name: "NamedLibrary", umdNamedDefine: true },
	},
	node: {
		__dirname: false,
		__filename: false
	}
};
