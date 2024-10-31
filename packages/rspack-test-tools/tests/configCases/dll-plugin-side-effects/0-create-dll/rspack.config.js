var path = require("path");
var rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: ["./index"],
	output: {
		filename: "dll.js",
		chunkFilename: "[id].dll.js",
		libraryTarget: "commonjs2"
	},
	module: {
		rules: [
			{
				test: /0-create-dll.(module|dependency)/,
				sideEffects: false
			}
		]
	},
	optimization: {
		usedExports: true,
		sideEffects: true,
		concatenateModules: false
	},
	plugins: [
		new rspack.DllPlugin({
			path: path.resolve(
				__dirname,
				"../../../js/config/dll-plugin-side-effects/manifest0.json",
			),
			entryOnly: false
		})
	]
};
