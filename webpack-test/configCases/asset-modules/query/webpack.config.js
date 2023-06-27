/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	output: {
		environment: {
			templateLiteral: false
		}
	},
	module: {
		rules: [
			{
				test: /\.(png|svg|jpg)$/,
				type: "asset/resource"
			}
		]
	}
};
