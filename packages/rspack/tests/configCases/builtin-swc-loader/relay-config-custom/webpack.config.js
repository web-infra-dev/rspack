module.exports = {
	module: {
		rules: [
			{
				test: /\.js$/,
				loader: "builtin:swc-loader",
				options: {
					rspackExperiments: {
						relay: {
							language: "typescript",
							artifactDirectory: "./custom"
						}
					}
				}
			}
		]
	}
};
