/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		chunkFilename: "[id].[hash].js"
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
