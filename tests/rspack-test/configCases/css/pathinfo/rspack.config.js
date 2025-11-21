/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	mode: "development",
	devtool: false,
	output: {
		pathinfo: true,
		cssChunkFilename: "[name].[chunkhash].css"
	},
	experiments: {
		css: true
	}
};
