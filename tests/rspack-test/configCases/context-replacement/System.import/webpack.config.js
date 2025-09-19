var path = require("path");
var webpack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new webpack.ContextReplacementPlugin(
			/context-replacement/,
			path.resolve(__dirname, "modules"),
			{
				a: "./module-b"
			}
		)
	]
};
