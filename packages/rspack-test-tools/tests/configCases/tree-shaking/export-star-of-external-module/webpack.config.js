/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	context: __dirname,
	module: {
		rules: []
	},
	builtins: {
		treeShaking: true
	},
	optimization: {
		sideEffects: true
	},
	externalsPresets: {
		node: true
	},
	externals: {
		"react-router-dom": "Buffer"
	}
};
