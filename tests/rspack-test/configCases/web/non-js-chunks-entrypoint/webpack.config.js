const { ProvideSharedPlugin } = require("@rspack/core").sharing;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		filename: "[name].js"
	},
	target: "web",
	optimization: {
		chunkIds: "named",
		splitChunks: {
			chunks: "all",
			minSize: 1,
			cacheGroups: {
				share: {
					type: "provide-module",
					name: "provide-module",
					enforce: true
				}
			}
		}
	},
	plugins: [
		new ProvideSharedPlugin({
			provides: ["package"]
		})
	]
};
