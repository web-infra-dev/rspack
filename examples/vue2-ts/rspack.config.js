const rspack = require("@rspack/core");
const { VueLoaderPlugin } = require("vue-loader");

/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	entry: {
		main: "./src/main.ts"
	},
	devServer: {
		historyApiFallback: true
	},
	devtool: false,
	plugins: [
		new VueLoaderPlugin(),
		new rspack.HtmlRspackPlugin({
			template: "./index.html"
		})
	],
	resolve: {
		extensions: [".vue", ".ts", "..."]
	},
	module: {
		rules: [
			{
				test: /\.vue$/,
				use: "vue-loader"
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
				use: ["vue-style-loader", "css-loader", "less-loader"],
				type: "javascript/auto"
			},
			{
				test: /\.css$/,
				use: ["vue-style-loader", "css-loader"],
				type: "javascript/auto"
			},
			{
				test: /\.svg$/,
				type: "asset/resource"
			}
		]
	}
};
module.exports = config;
