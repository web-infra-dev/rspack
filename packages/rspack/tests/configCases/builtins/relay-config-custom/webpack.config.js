module.exports = {
	builtins: {
		relay: {
			language: "typescript",
			artifactDirectory: "./custom"
		}
	},
	experiments: {
		rspackFuture: {
			disableTransformByDefault: false
		}
	}
};
