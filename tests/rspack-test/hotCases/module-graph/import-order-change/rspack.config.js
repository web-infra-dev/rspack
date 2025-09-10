const { rspack } = require("@rspack/core");

module.exports = {
	node: {
		__dirname: false,
		__filename: false
	},
	module: {
		rules: [
			{
				test: /\.css/,
				type: "javascript/auto",
				use: [rspack.CssExtractRspackPlugin.loader, "css-loader"]
			}
		]
	},
	plugins: [
		new rspack.CssExtractRspackPlugin({
			filename: "bundle.css"
		})
	],
	experiments: {
		css: false
	}
};
