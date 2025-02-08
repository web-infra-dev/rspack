const { rspack } = require("@rspack/core");
const { CssExtractRspackPlugin, HtmlRspackPlugin } = rspack;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		publicPath: "http://cdn.com/",
		crossOriginLoading: "anonymous"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [CssExtractRspackPlugin.loader, "css-loader"]
			}
		]
	},
	experiments: {
		css: false
	},
	plugins: [
		new CssExtractRspackPlugin(),
		new HtmlRspackPlugin({
			minify: false,
			template: "./index.html",
			title: "i am title",
			meta: {
				"meta-name": "meta-value"
			},
			inject: false,
			favicon: "./favicon.ico",
			templateParameters: {
				foo: "bar"
			}
		})
	]
};
