/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: 'web',
	node: false,
	entry: {
		main: "./index.js"
	},
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false,
				exportsConvention: "camel-case",
			}
		},
		rules: [
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	},

};
