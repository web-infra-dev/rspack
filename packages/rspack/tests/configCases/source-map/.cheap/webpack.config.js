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
	devtool: "cheap-source-map"
};
