module.exports = {
	devtool: "source-map",
	externals: ["source-map"],
	externalsType: "commonjs",
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
					}
				],
				type: "javascript/auto"
			},
			{
				resourceQuery: /resource/,
				type: "asset/resource",
				generator: {
					filename: "source.txt"
				}
			}
		]
	}
};
