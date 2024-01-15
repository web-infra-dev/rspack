module.exports = {
	module: {
		rules: [
			{
				test: /\.js$/,
				loader: "builtin:swc-loader",
				options: {
					rspackExperiments: {
						import: [
							{
								libraryName: "./lib",
								customName: "./lib/{{ unregisteredCase member }}"
							}
						]
					}
				}
			}
		]
	}
};
