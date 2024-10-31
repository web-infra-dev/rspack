var path = require("path");
var rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: ["./index.js"],
	output: {
		filename: "dll.js",
		chunkFilename: "[id].dll.js",
		library: {
			type: 'commonjs2',
		}
	},

	plugins: [
		new rspack.DllPlugin({
			path: path.resolve(
				__dirname,
				"../../../js/config/dll-plugin/issue-10475.json"
			)
		})
	]
};
