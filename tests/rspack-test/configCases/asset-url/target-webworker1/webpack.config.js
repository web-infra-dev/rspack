/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	target: "webworker",
	devtool: false,
	output: {
		filename: "deep/path/[name].js",
		assetModuleFilename: "[path][name][ext]",
		publicPath: ""
	}
};
