/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		assetModuleFilename: "assets/[name][ext]"
	},
	module: {
		rules: [
			{
				test: /\.png$/,
				use: "./loader.js",
				type: "asset/resource"
			}
		]
	}
};
