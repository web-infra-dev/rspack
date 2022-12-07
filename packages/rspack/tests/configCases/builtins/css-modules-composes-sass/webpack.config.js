module.exports = {
	module: {
		rules: [
			{
				test: /\.scss$/,
				use: [{ builtinLoader: "sass-loader" }],
				type: "css/module"
			}
		]
	}
};
