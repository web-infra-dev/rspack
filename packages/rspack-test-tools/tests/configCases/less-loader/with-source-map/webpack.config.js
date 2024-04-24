module.exports = {
	devtool: "source-map",
	module: {
		rules: [
			{
				test: /\.less$/,
				use: [{ loader: "less-loader" }],
				type: "css",
				generator: {
					exportsOnly: false,
				}
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
