/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		chunkFilename: "[id].[contenthash].js"
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
