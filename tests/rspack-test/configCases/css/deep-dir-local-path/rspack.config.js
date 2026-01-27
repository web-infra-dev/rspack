/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		generator: {
			"css/auto": {
				localIdentName: "[path][name]-[local]"
			}
		},
		rules: [
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	},
	mode: "development",

};
