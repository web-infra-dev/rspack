/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	context: __dirname,
	builtins: {
		treeShaking: true
	},
	optimization: {
		sideEffects: true
	}
};
