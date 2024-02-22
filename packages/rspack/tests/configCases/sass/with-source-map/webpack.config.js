const rspack = require("@rspack/core");
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
	devtool: "source-map",
	plugins: [
		new rspack.DefinePlugin({
			CONTEXT: JSON.stringify(__dirname)
		})
	]
};
