const rspack = require("@rspack/core");
/**
 * @type {import('@rspack/cli').Configuration}
 */
const config = {
	devtool: false,
	entry: {
		main: "./src/index",
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./index.html",
		}),
	],
	output: {
		clean: true,
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true,
		},
	},
	optimization: {
		concatenateModules: true,
	},
};

module.exports = config;
