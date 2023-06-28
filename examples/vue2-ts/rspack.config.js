const { VueLoaderPlugin } = require("vue-loader");

/** @type {import('@rspack/cli').Configuration} */
const config = {
	context: __dirname,
	entry: {
		main: "./src/main.ts"
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		]
	},
	devServer: {
		historyApiFallback: true
	},
	devtool: false,
	plugins: [new VueLoaderPlugin()],
	resolve: {
		extensions: [".vue", "..."]
	},
	module: {
		rules: [
			{
				test: /\.vue$/,
				use: ["vue-loader"]
			},
			{
				resourceQuery: /lang=ts/,
				type: "ts"
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
