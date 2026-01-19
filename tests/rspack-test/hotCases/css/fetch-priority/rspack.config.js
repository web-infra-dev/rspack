/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	devtool: false,
	module: {
		rules: [
			{
				test: /\.css$/,
				type: "css/auto"
			}
		]
	}
};
