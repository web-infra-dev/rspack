const { rspack } = require("@rspack/core");

module.exports = {
	context: __dirname,
	mode: "development",
	entry: {
		main: "./src/index.js"
	},
	devServer: {
		hot: true
	},
	stats: "none",
	infrastructureLogging: {
		debug: false
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./src/index.html"
		})
	],
	watchOptions: {
		poll: 1000
	}
};
