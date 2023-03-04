module.exports = {
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/i,
				use: [{ loader: "builtin:sass-loader" }],
				type: "css"
			}
		]
	},
	devtool: "source-map"
};
