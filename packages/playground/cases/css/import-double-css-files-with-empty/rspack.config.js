const rspack = require("@rspack/core");

module.exports = {
	context: __dirname,
	mode: "development",
	entry: "./src/index.js",
	devServer: {
		hot: true
	},
	cache: false,
	stats: "none",
	infrastructureLogging: {
		debug: false
	},
	watchOptions: {
		poll: 1000
	},
	plugins: [new rspack.HtmlRspackPlugin({ template: "./src/index.html" })]
};
