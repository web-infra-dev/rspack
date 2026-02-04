/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	devtool: 'eval',
	entry: "./index.js",
	output: {
		filename: "[contenthash].js"
	},
	stats: {
		all: false,
		assets: true
	}
};
