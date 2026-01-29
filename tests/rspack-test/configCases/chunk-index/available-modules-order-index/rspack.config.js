/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	node: false,
	entry: {
		main: "./main.js"
	},
	output: {
		filename: "[name].js"
	},
	optimization: {
		splitChunks: false,
		chunkIds: "named"
	},
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false
			}
		},
		rules: [
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	}
}