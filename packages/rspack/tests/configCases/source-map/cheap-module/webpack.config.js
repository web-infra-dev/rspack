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
	devtool: "cheap-module-source-map"
};
