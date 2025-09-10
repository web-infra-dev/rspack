const { HotModuleReplacementPlugin } = require("webpack");

/** @type {import("webpack").Configuration} */
module.exports = {
	entry: {
		main: "./src/a.js"
	},
	output: {
		filename: "[name].js",
		chunkLoading: "jsonp",
		enabledChunkLoadingTypes: ["jsonp"]
	},
	optimization: {
		runtimeChunk: {
			name: "bundle"
		}
	},
	target: "web",
	plugins: [new HotModuleReplacementPlugin()]
};
