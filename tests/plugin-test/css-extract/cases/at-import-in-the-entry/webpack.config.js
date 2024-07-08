const { CssExtractRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	entry: ["./a.css", "./b.css"],
	output: {
		pathinfo: true
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
			filename: "[name].css"
		})
	]
};
