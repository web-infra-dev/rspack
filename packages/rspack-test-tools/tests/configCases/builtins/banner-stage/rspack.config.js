const rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index",
		a: "./a"
	},
	target: "node",
	output: {
		filename: "[name].js",
		chunkFilename: "[name].js",
		assetModuleFilename: "[name][ext]"
	},
	devtool: "source-map",
	optimization: {
		chunkIds: "named"
	},
	module: {
		rules: [
			{
				test: /\.png/,
				type: "asset/resource"
			}
		]
	},
	plugins: [
		// Banner
		new rspack.BannerPlugin({
			banner: "PROCESS_ASSETS_STAGE_OPTIMIZE",
			entryOnly: true,
			exclude: [/a\.js/],
			stage: rspack.Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE // 100
		}),
		new rspack.BannerPlugin({
			banner: "PROCESS_ASSETS_STAGE_ADDITIONAL",
			entryOnly: true,
			exclude: [/a\.js/],
			stage: rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL // -2000
		}),
		new rspack.BannerPlugin("PROCESS_ASSETS_STAGE_ADDITIONS"), // Defaults to be PROCESS_ASSETS_STAGE_ADDITIONS(-100)

		// Fotter
		new rspack.BannerPlugin(
			{
				banner: "PROCESS_ASSETS_STAGE_REPORT",
				footer: true,
				entryOnly: true,
				exclude: [/a\.js/],
				stage: rspack.Compilation.PROCESS_ASSETS_STAGE_REPORT // 5000
			}
		),
		new rspack.BannerPlugin({
			banner: "PROCESS_ASSETS_STAGE_PRE_PROCESS",
			footer: true,
			entryOnly: true,
			exclude: [/a\.js/],
			stage: rspack.Compilation.PROCESS_ASSETS_STAGE_PRE_PROCESS // -1000
		}),
		new rspack.BannerPlugin({
			banner: "PROCESS_ASSETS_STAGE_DERIVED",
			footer: true,
			entryOnly: true,
			exclude: [/a\.js/],
			stage: rspack.Compilation.PROCESS_ASSETS_STAGE_DERIVED // -200
		}),
	]
};
