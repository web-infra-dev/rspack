/** @type {import("../../../../").Configuration} */
module.exports = {
	mode: "development",
	output: {
		assetModuleFilename: "file[ext]"
	},
	module: {
		rules: [
			{
				test: /\.png$/,
				type: "asset/resource",
				generator: {
					publicPath: "https://cdn/assets/",
					outputPath: "cdn-assets/"
				}
			}
		]
	}
};
