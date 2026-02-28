/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		uniqueName: "css-test"
	},
	module: {
		rules: [
			{
				test: /\.css/,
				type: "css/auto"
			}
		]
	}
};
