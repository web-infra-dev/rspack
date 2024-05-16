/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	mode: "production",
	context: __dirname,
	builtins: {
		treeShaking: true
	},
	optimization: {
		moduleIds: "named",
		minimize: false
	}
};
