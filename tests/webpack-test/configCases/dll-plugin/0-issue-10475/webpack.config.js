var path = require("path");
var webpack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: ["./index.js"],
	output: {
		filename: "dll.js",
		chunkFilename: "[id].dll.js",
		libraryTarget: "commonjs2"
	},
	plugins: [
		new webpack.DllPlugin({
			path: path.resolve(
				__dirname,
				"../../../js/config/dll-plugin/issue-10475.json"
			)
		})
	]
};
