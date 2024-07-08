/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	target: "node",
	entry: {
		main: "./index.js"
	},
	output: {
		chunkFilename: "[name].js"
	},
	optimization: {
		minimize: false,
		splitChunks: {
			minSize: 100
		}
	},
	externalsType: "commonjs",
	externals: [
		"./x_js-y_js.js",
		"./async-a.js",
		"./async-b.js",
		"./async-c.js",
		"./async-d.js",
		"./async-e.js",
		"./async-f.js",
		"./async-g.js"
	]
};
