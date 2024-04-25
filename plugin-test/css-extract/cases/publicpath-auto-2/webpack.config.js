const { CssExtractRspackPlugin } = require("@rspack/core");

module.exports = {
	entry: "./index.js",
	output: {
		publicPath: "auto"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: CssExtractRspackPlugin.loader
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
				test: /same_root\.(svg)$/,
				type: "asset/resource",
				generator: { filename: "[name][ext]" }
			},
			{
				test: /same_dir\.(svg)$/,
				type: "asset/resource",
				generator: { filename: "styles/[name][ext]" }
			},
			{
				test: /nested_dir\.(svg)$/,
				type: "asset/resource",
				generator: { filename: "styles/nested/[name][ext]" }
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
			filename: "styles/[name].css"
		})
	]
};
