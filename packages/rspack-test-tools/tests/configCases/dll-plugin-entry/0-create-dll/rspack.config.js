var path = require("path");
var rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: ["."],
	output: {
		filename: "dll.js",
		chunkFilename: "[id].dll.js",
		libraryTarget: "commonjs2"
	},
	plugins: [
		new rspack.DllPlugin({
			path: path.resolve(
				__dirname,
				"../../../js/config/dll-plugin-entry/manifest0.json"
			)
		})
	]
};
