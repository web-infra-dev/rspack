const { CssExtractRspackPlugin } = require("../../../../");

module.exports = [
	{
		entry: "./index.js",
		output: {
			filename: "one-[name].js"
		},
		module: {
			rules: [
				{
					test: /\.css$/,
					use: [CssExtractRspackPlugin.loader, "css-loader"]
				}
			]
		},
		plugins: [
			new CssExtractRspackPlugin({
				filename: "one/[name].css"
			})
		]
	},
	{
		entry: "./index.js",
		output: {
			filename: "two-[name].js"
		},
		module: {
			rules: [
				{
					test: /\.css$/,
					use: [CssExtractRspackPlugin.loader, "css-loader"]
				}
			]
		},
		plugins: [
			new CssExtractRspackPlugin({
				filename: "two/[name].css"
			})
		]
	}
];
