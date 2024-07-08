const rspack = require("@rspack/core");

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
	context: __dirname,
	mode: "development",
	entry: "./src/index.js",
	output: {
		cssFilename: "css/[name].css"
	},
	devtool: false,
	devServer: {
		hot: true
	},
	stats: "none",
	infrastructureLogging: {
		debug: false
	},
	plugins: [new rspack.HtmlRspackPlugin()],
	watchOptions: {
		poll: 1000
	},
	experiments: {
		css: true
	}
};
