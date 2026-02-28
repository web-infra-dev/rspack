/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index.js"
	},
	module: {
		parser: {
			"css/module": {
				namedExports: false
			}
		},
		rules: [
			{
				test: /\.module\.css$/,
				type: "css/module",
				generator: {
					exportsOnly: true
				}
			}
		]
	}
};
