import { RspackCssExtractPlugin } from "../../../../src";

module.exports = {
	entry: {
		entry1: { import: ["./entryA.js", "./entryB.js"], dependOn: "common" },
		common: ["./entryC.js", "./entryD.js"]
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
