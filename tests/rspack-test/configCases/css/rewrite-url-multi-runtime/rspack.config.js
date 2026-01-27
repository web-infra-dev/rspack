/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		a: "./index.js",
		b: "./index.js"
	},
	output: {
		filename: "[name].js",
		cssFilename: "bundle.css"
	},
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
				type: "asset/resource"
			}
		]
	},
	experiments: {
		css: true
	}
};
