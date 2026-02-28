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
				test: /\.png$/,
				type: "asset"
			},
			{
				test: /\.css/,
				type: "css/auto"
			}
		]
	}
};
