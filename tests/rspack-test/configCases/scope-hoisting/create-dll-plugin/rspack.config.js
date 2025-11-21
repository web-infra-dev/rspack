const path = require("path");
var webpack = require("@rspack/core");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: ["./index.js"],
	// CHANGE: use optimization.concatenateModules instead of ModuleConcatenationPlugin
	optimization: {
		concatenateModules: true
	},
	plugins: [
		new webpack.DllPlugin({
			path: path.resolve(
				__dirname,
				"../../../js/config/scope-hoisting/create-dll-plugin/manifest.json"
			)
		}),
		// new webpack.optimize.ModuleConcatenationPlugin()
	]
};
