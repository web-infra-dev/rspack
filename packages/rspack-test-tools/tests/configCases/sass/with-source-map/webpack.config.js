const rspack = require("@rspack/core");
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
	devtool: "source-map",
	plugins: [
		new rspack.DefinePlugin({
			CONTEXT: JSON.stringify(__dirname)
		})
	]
};
