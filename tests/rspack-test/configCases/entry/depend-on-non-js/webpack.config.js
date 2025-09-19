// CHANGE:
// const MiniCssPlugin = require("mini-css-extract-plugin");
const MiniCssPlugin = require("@rspack/core").CssExtractRspackPlugin;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		a: "./a.js",
		b: { import: "./b.js", dependOn: "a" }
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				loader: MiniCssPlugin.loader
			}
		]
	},
	output: {
		filename: "[name].js"
	},
	optimization: {
		runtimeChunk: "single",
		splitChunks: {
			chunks: "all",
			cacheGroups: {
				styles: {
					type: "css/mini-extract",
					enforce: true
				}
			}
		}
	},

	target: "web",
	plugins: [
		new MiniCssPlugin({
			experimentalUseImportModule: true
		})
	],
	experiments: {
		css: false
	},
};
