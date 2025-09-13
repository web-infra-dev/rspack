/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	mode: "development",
	experiments: {
		css: true
	},
	module: {
		rules: [
			{
				test: /\.less$/,
				use: "less-loader",
				type: "css/auto"
			}
		]
	}
};
