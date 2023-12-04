module.exports = {
	entry: {
		main: "./src/index.js"
	},
	builtins: {
		html: [
			{
				template: "./src/index.html"
			}
		]
	},
	optimization: {
		chunkIds: "named"
	},
	output: { clean: true },

	experiments: {
		incrementalRebuild: {
			emitAsset: true
		}
	}
};
