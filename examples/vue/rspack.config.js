const rspack = require("@rspack/core");
const { VueLoaderPlugin } = require("vue-loader");

/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	entry: {
		main: "./src/main.js"
	},
	devServer: {
		historyApiFallback: true
	},
	plugins: [
		new VueLoaderPlugin(),
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		}),
		new rspack.DefinePlugin({
			__VUE_OPTIONS_API__: JSON.stringify(true),
			__VUE_PROD_DEVTOOLS__: JSON.stringify(false)
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
				test: /\.ts$/,
				loader: "builtin:swc-loader",
				options: {
					sourceMap: true,
					jsc: {
						parser: {
							syntax: "typescript"
						}
					}
				},
				type: "javascript/auto"
			},
			{
				test: /\.less$/,
				loader: "less-loader",
				type: "css"
			},
			{
				test: /\.svg$/,
				type: "asset/resource"
			}
		]
	}
};
module.exports = config;
