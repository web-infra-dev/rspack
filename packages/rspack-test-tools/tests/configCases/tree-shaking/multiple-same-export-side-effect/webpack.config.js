const { DefinePlugin } = require("@rspack/core");

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	context: __dirname,
	builtins: {
		treeShaking: true
	},
	optimization: {
		sideEffects: true
	},
	plugins: [
		new DefinePlugin({
			"process.env.NODE_ENV": JSON.stringify("production")
		})
	]
};
