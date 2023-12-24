const { HotModuleReplacementPlugin } = require("webpack");

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
