/** @type {import("../../../../src/index").RspackOptions} */
module.exports = {
	entry: {
		main: "./index"
	},
	target: "node",
	output: {
		filename: "[name].js",
		chunkFilename: "chunks/async-[name].[chunkhash:8].js"
	}
};
