/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		hashDigestLength: 8
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: [
					"style-loader",
					{
						loader: "css-loader",
						options: {
							modules: {
								localIdentName: "[name]__[local]--[contenthash]"
							}
						}
					}
				]
			}
		]
	},
	experiments: {
		css: false
	}
};
