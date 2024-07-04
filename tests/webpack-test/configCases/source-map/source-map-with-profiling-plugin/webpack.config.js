var webpack = require("@rspack/core");
var path = require("path");
var os = require("os");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	node: {
		__dirname: false,
		__filename: false
	},
	entry: {
		bundle0: ["./index.js"]
	},
	output: {
		filename: "[name].js"
	},
	plugins: [
		new webpack.debug.ProfilingPlugin({
			outputPath: path.join(os.tmpdir(), "events.json")
		})
	],
	devtool: "source-map"
};
