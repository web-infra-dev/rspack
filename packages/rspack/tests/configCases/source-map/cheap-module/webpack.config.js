module.exports = {
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false
			},
		},
		rules: [
			{
				test: /\.s[ac]ss$/i,
				use: [{ loader: "sass-loader" }],
				type: "css"
			}
		]
	},
	devtool: "cheap-module-source-map",
	externals: ["source-map"],
	externalsType: "commonjs"
};
