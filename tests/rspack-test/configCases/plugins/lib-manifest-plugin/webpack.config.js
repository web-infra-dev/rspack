var path = require("path");
var LibManifestPlugin = require("@rspack/core").LibManifestPlugin;

/** @type {function(any, any): import("@rspack/core").Configuration} */
module.exports = (env, { testPath }) => ({
	entry: {
		bundle0: ["./"]
	},
	plugins: [
		new LibManifestPlugin({
			path: path.resolve(testPath, "[name]-manifest.json"),
			name: "[name]_[fullhash]"
		})
	],
	node: {
		__dirname: false
	}
});
