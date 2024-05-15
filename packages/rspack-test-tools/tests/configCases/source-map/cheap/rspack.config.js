/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /\.s[ac]ss$/i,
				use: [{ loader: "sass-loader" }],
				type: "css",
				generator: {
					exportsOnly: false,
				}
			}
		]
	},
	devtool: "cheap-source-map",
	externals: ["source-map"],
	externalsType: "commonjs"
};
