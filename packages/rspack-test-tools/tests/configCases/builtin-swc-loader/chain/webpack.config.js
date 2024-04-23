module.exports = {
	resolve: {
		extensions: ["...", ".ts", ".tsx", ".jsx"]
	},
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
	}
};
