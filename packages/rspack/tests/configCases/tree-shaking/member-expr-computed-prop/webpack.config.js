/**@type {import('@rspack/cli').Configuration}*/
module.exports = {
	context: __dirname,
	builtins: {
		treeShaking: true
	},
	optimization: {
		sideEffects: true
	}
};
