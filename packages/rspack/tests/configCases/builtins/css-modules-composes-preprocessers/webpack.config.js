module.exports = {
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/module"
			},
			{
				test: /\.scss$/,
				use: [{ loader: "builtin:sass-loader" }],
				type: "css/module"
			},
			{
				test: /\.less$/,
				use: [{ loader: "less-loader" }],
				type: "css/module"
			}
		]
	}
};
