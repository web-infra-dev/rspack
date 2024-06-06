/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		assetModuleFilename: "[name][ext][query][fragment]",
		publicPath: "public/"
	},
	module: {
		parser: {
			javascript: {
				url: "relative"
			}
		}
	}
};
