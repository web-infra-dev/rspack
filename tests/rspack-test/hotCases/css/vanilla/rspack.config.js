/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	devtool: false,
	output: {
		cssChunkFilename: "[name].css"
	},
	node: {
		__dirname: false
	},
	experiments: {
		css: true
	}
};
