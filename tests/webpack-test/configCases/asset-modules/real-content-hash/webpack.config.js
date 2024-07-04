/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	output: {
		publicPath: "assets/",
		assetModuleFilename: "file[ext]"
	},
	module: {
		rules: [
			{
				test: /\.png$/,
				type: "asset/resource"
			}
		]
	}
};
