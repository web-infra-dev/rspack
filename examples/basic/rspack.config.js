const rspack = require("@rspack/core");
/**@type {import("@rspack/cli").Configuration}*/
module.exports = {
	mode: 'production',
	experiments: {
		rspackFuture: {
			newTreeshaking: true,
		},
		css: true,
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/module",
			},
		],
	},
	optimization: {
		concatenateModules: false,
		minimize: false,
	},
	plugins: [
		new rspack.HtmlRspackPlugin({
			template: "./index.html",
		}),
	],
};
