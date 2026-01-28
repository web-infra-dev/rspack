/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	entry: {
		main: "./index"
	},
	output: {
		filename: "[name].js"
	},
	node: {
		__dirname: false
	},
	optimization: {
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
				type: 'css/auto'
			}
		]
	}
};
