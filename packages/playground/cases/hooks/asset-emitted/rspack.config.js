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
	experiments: {
		incrementalRebuild: {
			emitAsset: true
		}
	}
};
