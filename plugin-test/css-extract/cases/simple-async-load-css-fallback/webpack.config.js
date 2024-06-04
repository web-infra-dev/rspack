const { CssExtractRspackPlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index.js"
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
		// function Plugin() {
		// 	this.hooks.compilation.tap("Test", compilation => {
		// 		compilation.hooks.beforeChunkAssets.tap("Test", () => {
		// 			for (const chunkGroup of compilation.chunkGroups) {
		// 				// remove getModuleIndex2 to enforce using fallback
		// 				// eslint-disable-next-line no-undefined
		// 				chunkGroup.getModuleIndex2 = undefined;
		// 			}
		// 		});
		// 	});
		// },
		new CssExtractRspackPlugin({
			filename: "[name].css"
		})
	]
};
