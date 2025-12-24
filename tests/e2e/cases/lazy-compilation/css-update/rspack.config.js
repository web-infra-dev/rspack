const { rspack } = require("@rspack/core");

/** @type { import('@rspack/core').RspackOptions } */
module.exports = {
	context: __dirname,
	entry: "./src/index.js",
	mode: "development",
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [rspack.CssExtractRspackPlugin.loader, "css-loader"]
			}
		]
	},
	plugins: [new rspack.HtmlRspackPlugin(), new rspack.CssExtractRspackPlugin()],
	experiments: {
		css: false
	},
	lazyCompilation: true,
	optimization: {
		splitChunks: {
			minSize: 0,
			chunks: "all",
			cacheGroups: {
				styles: {
					test: /\.css$/,
					name: "style.css"
				}
			}
		}
	},
	devServer: {
		hot: true,
		port: 5678
	}
};
