/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	node: false,
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false
			}
		},
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: "builtin:lightningcss-loader",
						/** @type {import("@rspack/core").LightningcssLoaderOptions} */
						options: {
							targets: ["Edge >= 12", "iOS >= 8", "Android >= 4.0"]
						}
					}
				],
				type: "css/auto"
			}
		]
	},

};
