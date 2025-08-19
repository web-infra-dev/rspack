/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "node",
	entry: "./index.js",
	output: {
		filename: "[name].js"
	},
	optimization: {
		splitChunks: {
			minSize: 1,
			cacheGroups: {
				vendors: {
					name: "vendors",
					test: /[\\/]node_modules[\\/]/,
					priority: 10,
					enforce: true
				},
				vendorsCommon: {
					test: /[\\/]node_modules[\\/]/,
					name: "vendors-common",
					minChunks: 2,
					priority: 12,
					enforce: true
				}
			}
		}
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.done.tap("PLUGIN", stats => {
					const json = stats.toJson({
						all: true
					});
					expect(
						json.assets.find(asset => asset.name === "vendors.js")
					).toBeTruthy();
					expect(
						json.assets.find(asset => asset.name === "vendors-common.js")
					).toBeTruthy();
				});
			}
		}
	],
	stats: {
		all: true
	}
};
