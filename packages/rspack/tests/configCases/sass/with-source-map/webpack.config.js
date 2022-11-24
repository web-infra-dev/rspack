module.exports = {
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/i,
				uses: [{ builtinLoader: "sass-loader" }],
				type: "css"
			}
		]
	},
	devtool: "source-map"
};
