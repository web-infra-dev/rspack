const { CssExtractRspackPlugin } = require("@rspack/core");

module.exports = {
	entry: "./index.js",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: CssExtractRspackPlugin.loader,
						options: {
							publicPath: "auto"
						}
					},
					"css-loader"
				]
			},
			{
				test: /outer\.(svg)$/,
				type: "asset/resource",
				generator: { filename: "../[name][ext]" }
			},
			{
				test: /img1\.(svg)$/,
				type: "asset/resource",
				generator: { filename: "[name][ext]" }
			},
			{
				test: /img2\.(svg)$/,
				type: "asset/resource",
				generator: { filename: "assets/[name][ext]" }
			},
			{
				test: /img3\.(svg)$/,
				type: "asset/resource",
				generator: { filename: "assets/nested/[name][ext]" }
			},
			{
				test: /img4\.(svg)$/,
				type: "asset/resource",
				generator: { filename: "[name][ext]" }
			},
			{
				test: /react\.(svg)$/,
				type: "asset/resource",
				generator: { filename: "assets/img/[name][ext]" }
			}
		]
	},
	plugins: [
		new CssExtractRspackPlugin({
			filename: "[name].css"
		})
	]
};
