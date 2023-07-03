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
	output: { clean: true },
	experiments: {
		incrementalRebuild: {
			emitAsset: true
		}
	}
};
