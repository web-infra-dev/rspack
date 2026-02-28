/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	node: false,
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false
			}
		},
		rules: [
			{
				test: /\.css/,
				type: "css/auto"
			}
		]
	},
	optimization: {
		minimize: true,
		minimizer: []
	}
};
