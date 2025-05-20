const { CopyRspackPlugin } = require("@rspack/core");
const path = require("path");

module.exports = {
	entry: "./index.js",
	target: "node",
	plugins: [
		new CopyRspackPlugin({
			patterns: [
				{
					from: path.join(__dirname, "src", "test-sync-fn.txt"),
					copyPermissions: true,
					transform: (content, absoluteFilename) => {
						return `file: ${absoluteFilename} transformed: ${content} changed`;
					}
				},
				{
					from: path.join(__dirname, "src", "test-async-fn.txt"),
					copyPermissions: true,
					transform: async (content, absoluteFilename) => {
						return `file: ${absoluteFilename} transformed: ${content} changed`;
					}
				},
				{
					from: path.join(__dirname, "src", "test-sync-obj.txt"),
					copyPermissions: true,
					transform: {
						transformer: (content, absoluteFilename) => {
							return `file: ${absoluteFilename} transformed: ${content} changed`;
						}
					}
				},
				{
					from: path.join(__dirname, "src", "test-async-obj.txt"),
					copyPermissions: true,
					transform: {
						transformer: async (content, absoluteFilename) => {
							return `file: ${absoluteFilename} transformed: ${content} changed`;
						}
					}
				}
			]
		})
	]
};
