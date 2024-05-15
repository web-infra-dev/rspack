/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		publicPath: "/",
		cssFilename: "css/[name].css"
	},
	resolve: {
		alias: {
			"@": __dirname
		}
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
