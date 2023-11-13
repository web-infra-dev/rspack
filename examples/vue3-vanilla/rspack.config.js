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
	optimization: {
		minimize: false, // Disabling minification because it takes too long on CI
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
				loader: "vue-loader"
			},
			{
				test: /\.less$/,
				use: [
					{
						loader: "style-loader",
						options: {
							esModule: false
						}
					},
					"css-loader",
					"less-loader"
				],
				type: "javascript/auto"
			},
			{
				test: /\.css$/,
				use: [
					{
						loader: "style-loader",
						options: {
							esModule: false
						}
					},
					"css-loader"
				],
				type: "javascript/auto"
			},
			{
				test: /\.svg$/,
				type: "asset/resource"
			}
		]
	},
	experiments: {
		css: false
	}
};
module.exports = config;
