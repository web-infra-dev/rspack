/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	node: {
		__dirname: false,
		__filename: false
	},
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false
			}
		},
		rules: [
			{
				test: /\.png$/i,
				type: "asset"
			},
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	},

};
