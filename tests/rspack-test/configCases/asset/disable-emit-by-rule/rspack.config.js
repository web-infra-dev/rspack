/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	output: {
		assetModuleFilename: "images/file[ext]"
	},
	module: {
		rules: [
			{
				test: /\.png$/,
				type: "asset/resource",
				generator: {
					emit: false
				}
			},
			{
				test: /\.jpg$/,
				type: "asset/resource"
			}
		]
	}
};
