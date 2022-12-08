module.exports = {
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/module"
			},
			{
				test: /\.scss$/,
				use: [{ builtinLoader: "sass-loader" }],
				type: "css/module"
			},
			{
				test: /\.less$/,
				use: [{ loader: "@rspack/less-loader" }],
				type: "css/module"
			}
		]
	}
};
