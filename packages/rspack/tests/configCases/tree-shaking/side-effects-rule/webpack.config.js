/**@type {import('@rspack/cli').Configuration}*/
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /package/,
				sideEffects: false
			}
		]
	},
	builtins: {
		treeShaking: true
	},
	optimization: {
		sideEffects: true
	},
	externalsPresets: {
		node: true
	}
};
