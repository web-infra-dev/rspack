import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: {
		entry1: { import: "./entryA.js", dependOn: "common" },
		common: "./entryB.js"
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [RspackCssExtractPlugin.loader, "css-loader"]
			}
		]
	},
	plugins: [
		new RspackCssExtractPlugin({
			filename: "[name].css"
		})
	]
};
