/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	entry: "./index.js",
	experiments: {
		css: true
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	},
	target: "web"
};