const rspack = require("@rspack/core");

module.exports = {
	entry: {
		main: "./index",
		a: "./a"
	},
	output: {
		assetModuleFilename: "[name][ext]"
	},
	devtool: "source-map",
	module: {
		rules: [
			{
				test: /\.png/,
				type: "asset/resource"
			}
		]
	},
	plugins: [
		new rspack.BannerPlugin("MMMMMMM"),
		new rspack.BannerPlugin({
			banner: "/** MMMMMMM */",
			raw: true,
			footer: true,
			entryOnly: true,
			exclude: [/a\.js/]
		})
	]
};
