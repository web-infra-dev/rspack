/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	output: {
		assetModuleFilename: "images/file[ext]"
	},
	module: {
		generator: {
			asset: {
				emit: false,
			},
		},
		parser: {
			asset: {
				dataUrlCondition: {
					maxSize: 0
				}
			}
		},
		rules: [
			{
				test: /\.png$/,
				type: "asset",
			},
			{
				test: /\.jpg$/,
				type: "asset"
			}
		]
	}
};
