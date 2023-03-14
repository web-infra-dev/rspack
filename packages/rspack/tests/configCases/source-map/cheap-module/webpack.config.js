module.exports = {
	module: {
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
