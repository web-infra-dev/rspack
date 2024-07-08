/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false,
			}
		},
		rules: [
			{
				test: /\.png$/i,
				type: "asset"
			}
		]
	},
	experiments: {
		css: true
	}
};
