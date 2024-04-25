const { CssExtractRspackPlugin } = require("@rspack/core");

module.exports = {
	entry: {
		entry1: { import: ["./entryA.js", "./entryB.js"], dependOn: "common" },
		common: ["./entryC.js", "./entryD.js"]
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
