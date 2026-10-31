const { HotModuleReplacementPlugin } = require("@rspack/core");

module.exports = ["webpack", "rspack"].map(runtimeMode => ({
	mode: "development",
	experiments: { runtimeMode },
	plugins: [new HotModuleReplacementPlugin()],
	optimization: {
		chunkIds: "named",
		splitChunks: {
			minSize: 0,
			cacheGroups: {
				x: { test: /[\\/]x\.js$/, name: "x", enforce: true, priority: 10 },
				shared: { test: /[\\/][ab]\.js$/, name: "shared", enforce: true }
			}
		}
	}
}));
