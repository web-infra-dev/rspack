const { rspack } = require("@rspack/core");

module.exports = {
	entry: {
		main: "./src/index.js"
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./src/index.html"
		})
	],
	optimization: {
		chunkIds: "named"
	},
	output: { clean: true }
};
