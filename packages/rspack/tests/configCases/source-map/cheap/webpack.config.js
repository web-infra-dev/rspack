module.exports = {
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/i,
				use: [{ builtinLoader: "sass-loader" }],
				type: "css"
			}
		]
	},
	devtool: "cheap-source-map"
};
