/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	node: false,
	entry: {
		main: "./index.js"
	},
	module: {
		rules: [
			{
				test: /\.css/,
				type: "css/auto"
			}
		],
		parser: {
			"css/auto": {
				resolveImport: false
			}
		},
		generator: {
			"css/auto": {
				exportsOnly: false
			}
		}
	},
};
