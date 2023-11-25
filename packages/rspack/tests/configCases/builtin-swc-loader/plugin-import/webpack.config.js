module.exports = {
	module: {
		rules: [
			{
				test: /.css/,
				type: "asset"
			}
		]
	},
	module: {
		rules: [
			{
				test: /\.js$/,
				loader: "builtin:swc-loader",
				options: {
					rspackExperiments: {
						import: [
							{
								libraryName: "./src/foo",
								customName: "./src/foo/{{ kebabCase member }}",
								style: true
							},
							{
								libraryName: "./src/bar",
								customName: "./src/bar/{{ kebabCase member }}",
								style: `{{ member }}/style.css`
							}
						]
					}
				}
			}
		]
	}
};
