/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "web",
	mode: "development",
	module: {
		rules: [
			{
				test: /\.less$/,
				use: "less-loader",
				type: "css/auto"
			}
		]
	},
	experiments: {
		css: true
	}
};
