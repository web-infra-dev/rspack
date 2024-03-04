const minifyPlugin = require("@rspack/plugin-minify");
const { SourceMapDevToolPlugin } = require('@rspack/core')

module.exports = {
	context: __dirname,
	target: "node",
	entry: {
		main: "./index.js"
	},
	optimization: {
		minimize: true,
		minimizer: [new minifyPlugin({})]
	},
	plugins: [
		new SourceMapDevToolPlugin({})
	],
	devtool: false,
};
