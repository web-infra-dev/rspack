module.exports = {
	module: {
		rules: [
			{
				test: /\.jsx?$/,
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
