const path = require("path");
const rspack = require("@rspack/core");

/**
 * @type {import("@rspack/core").Configuration}
 */
module.exports = {
	node: {
		__dirname: false,
		__filename: false
	},
	output: {
		filename: "[name].js"
	},
	plugins: [
		new rspack.SourceMapDevToolPlugin({
			filename: "[file].map",
			sourceRoot: path.join(__dirname, "folder") + "/"
		}),
		new rspack.DefinePlugin({
			CONTEXT: JSON.stringify(__dirname)
		})
	]
};
