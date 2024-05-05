/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		cssFilename: "css/[name].css"
	},
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false,
			}
		},
		rules: [
			{
				test: /\.png$/i,
				type: "asset",
				parser: {
					dataUrlCondition: {
						maxSize: 30000
					}
				},
				generator: {
					filename: "image/[name].[contenthash:8][ext]"
				}
			}
		]
	},
	experiments: {
		css: true
	}
};
