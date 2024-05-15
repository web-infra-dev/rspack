/**@type {import("@rspack/core").Configuration}*/
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
		treeShaking: true
	},
	optimization: {
		sideEffects: true
	},
	externalsPresets: {
		node: true
	}
};
