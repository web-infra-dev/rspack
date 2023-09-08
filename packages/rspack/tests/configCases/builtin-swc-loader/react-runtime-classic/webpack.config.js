module.exports = {
	module: {
		rules: [
			{
				test: /\.js$/,
				loader: "builtin:swc-loader",
				options: {
					rspackExperiments: {
						react: {
							runtime: "classic"
						}
					}
				}
			}
		]
	}
};
