/** @type {import("../../../../src/index").RspackOptions} */
module.exports = {
	entry: {
		main: "./index"
	},
	devtool: "source-map",
	output: {
		filename: "[name].js",
		sourceMapFilename: "[name].js.map"
	}
};
