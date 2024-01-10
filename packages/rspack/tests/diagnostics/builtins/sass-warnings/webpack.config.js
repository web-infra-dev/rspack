module.exports = {
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/,
				loader: "builtin:sass-loader",
				type: "css"
			}
		]
	}
}
