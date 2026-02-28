/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	node: {
		__dirname: false,
		__filename: false
	},
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
				exportsOnly: false
			}
		},
		rules: [
			{
				test: /\.png$/i,
				type: "asset",
				generator: {
					filename: "image/[name].[contenthash:8][ext]"
				}
			},
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	},

};
