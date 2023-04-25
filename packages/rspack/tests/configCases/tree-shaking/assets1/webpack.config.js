/**@type {import('@rspack/cli')}*/
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /\.svg$/,
				type: "asset/resource"
			}
		]
	},
	builtins: {
		treeShaking: false
	},
	optimization: {
		sideEffects: false
	}
};
