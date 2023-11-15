const rspack = require("@rspack/core");
const { VueLoaderPlugin } = require("vue-loader");

/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	entry: {
		main: "./src/main.js"
	},
	plugins: [
		new VueLoaderPlugin(),
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		})
	],
	module: {
		rules: [
			{
				test: /\.vue$/,
				loader: "vue-loader",
				options: {
					experimentalInlineMatchResource: true
				}
			},
			{
				test: /\.svg/,
				type: "asset/resource"
			}
		]
	}
};
module.exports = config;
