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
	builtins: {
		banner: [
			"MMMMMMM",
			{
				banner: "/** MMMMMMM */",
				raw: true,
				footer: true,
				entryOnly: true,
				exclude: [/a\.js/]
			}
		]
	}
};
