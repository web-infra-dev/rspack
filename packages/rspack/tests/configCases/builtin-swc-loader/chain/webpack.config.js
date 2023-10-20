module.exports = {
	module: {
		rules: [
			{
				test: /\.ts$/,
				use: [
					{
						loader: "builtin:swc-loader",
						options: {
							jsc: {
								parser: {
									syntax: "typescript"
								}
							}
						}
					},
					"./loader.js",
					{
						loader: "builtin:swc-loader",
						options: {
							jsc: {
								parser: {
									syntax: "typescript"
								}
							}
						}
					}
				],
				type: "javascript/auto"
			}
		]
	},
	experiments: {
		rspackFuture: {
			disableTransformByDefault: true
		}
	}
};
