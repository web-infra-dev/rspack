var webpack = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	// CHANGE: use optimization.concatenateModules instead of ModuleConcatenationPlugin
	optimization: {
		concatenateModules: true
	},
	plugins: [
		new webpack.DllReferencePlugin({
			name: "function(id) { return {default: 'ok'}; }",
			scope: "dll",
			content: {
				"./module": {
					id: 1,
					buildMeta: {
						exportsType: "namespace"
					},
					exports: ["default"]
				}
			}
		}),
		// new webpack.optimize.ModuleConcatenationPlugin()
	]
};
