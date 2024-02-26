/** @type {import("../../../../src/index").RspackOptions} */
module.exports = {
	entry: {
		main: "./index"
	},
	devtool: "source-map",
	target: "node",
	output: {
		filename: "[name].js",
		sourceMapFilename: "[name].js.map"
	}
};
