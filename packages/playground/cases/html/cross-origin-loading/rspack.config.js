const rspack = require("@rspack/core");

module.exports = {
	context: __dirname,
	mode: "development",
	entry: "./src/index.js",
	stats: "none",
	output: {
		crossOriginLoading: "anonymous"
	},
	devServer: {
		port: 3000
	},
	plugins: [new rspack.HtmlRspackPlugin({ template: "./src/index.html" })]
};
