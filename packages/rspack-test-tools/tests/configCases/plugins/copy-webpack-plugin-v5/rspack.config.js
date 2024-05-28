const CopyPlugin = require("copy-webpack-plugin");
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new CopyPlugin([
			{
				from: "*.txt",
				to: "[name].[ext]",
				transform(content) {
					return content + "to";
				}
			}
		])
	]
};
