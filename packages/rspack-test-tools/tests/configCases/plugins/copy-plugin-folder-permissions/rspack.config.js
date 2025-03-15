const { CopyRspackPlugin } = require("@rspack/core");
const path = require("path");

module.exports = {
	entry: "./index.js",
	target: "node",
	plugins: [
		new CopyRspackPlugin({
			patterns: [
				{
					from: path.join(__dirname, "src"),
					to: path.join(__dirname, "dist"),
					copyPermissions: true
				}
			]
		})
	]
};
