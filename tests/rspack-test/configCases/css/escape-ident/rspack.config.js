/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false,
				localIdentName: "[local]-[path]"
			}
		},
		rules: [
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	}
};
