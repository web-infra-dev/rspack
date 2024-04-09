module.exports = {
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/i,
				use: [{ loader: "sass-loader" }],
				type: "css",
				generator: {
					exportsOnly: false
				},
			}
		]
	},
	devtool: "cheap-module-source-map",
	externals: ["source-map"],
	externalsType: "commonjs"
};
