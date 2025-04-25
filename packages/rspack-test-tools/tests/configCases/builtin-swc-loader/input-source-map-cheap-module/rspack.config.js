const { DefinePlugin } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	devtool: "cheap-module-source-map",
	resolve: {
		extensions: ["...", ".jsx"]
	},
	module: {
		rules: [
			{
				test: /a\.jsx$/,
				use: [
					{
						loader: "builtin:swc-loader",
						options: {
							sourceMaps: true
						}
					},
					"./prev-loader"
				]
			}
		]
	},
	plugins: [
		new DefinePlugin({
			CONTEXT: JSON.stringify(__dirname)
		})
	]
};
