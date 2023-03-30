/** @type {import("@rspack/cli").Configuration} */
module.exports = {
	target: "web",
	output: {
		chunkFilename: "[name].js",
		uniqueName: 'my "app"'
	},
	performance: {
		hints: false
	},
	optimization: {
		chunkIds: "named",
		minimize: false
	}
};
