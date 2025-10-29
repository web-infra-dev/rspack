/** @type {import("@rspack/core").Configuration} */
module.exports = {
	node: {
		__dirname: false,
		__filename: false
	},
	mode: "production",
	optimization: {
		mangleExports: "size"
	},
	experiments: {
		inlineConst: false
	}
};
