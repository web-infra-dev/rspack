var HotModuleReplacementPlugin =
	require("@rspack/core").HotModuleReplacementPlugin;
/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index"
	},
	target: "web",
	output: {
		filename: "[name].js"
	},
	optimization: {
		splitChunks: {
			cacheGroups: {
				vendor: {
					chunks: "all",
					name: "vendor",
					test: /vendor/,
					enforce: true
				}
			}
		}
	},
	plugins: [new HotModuleReplacementPlugin()]
};
