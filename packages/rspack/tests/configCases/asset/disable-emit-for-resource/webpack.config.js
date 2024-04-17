/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	output: {
		assetModuleFilename: "images/file[ext]"
	},
	module: {
		generator: {
			'asset/resource': {
				emit: false,
			},
		},
		rules: [
			{
				test: /\.png$/,
				type: "asset/resource",
			},
			{
				test: /\.jpg$/,
				type: "asset/resource"
			}
		]
	}
};
