module.exports = {
	module: {
		rules: [
			{
				test: /\.scss$/,
				use: [{ loader: "builtin:sass-loader" }],
				type: "css/module"
			}
		]
	}
};
