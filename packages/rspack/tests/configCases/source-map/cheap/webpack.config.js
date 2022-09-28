module.exports = {
	module: {
		rules: [
			{
				test: "\\.s[ac]ss$",
				uses: [{ builtinLoader: "sass-loader" }],
				type: "css"
			}
		]
	},
	devtool: "cheap-source-map"
};
