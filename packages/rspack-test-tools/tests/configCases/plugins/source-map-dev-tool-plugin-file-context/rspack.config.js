const { rspack } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
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
			filename: "sourcemaps/[file].map",
			fileContext: "assets",
			publicPath: "http://localhost:50505/"
		})
	]
};
