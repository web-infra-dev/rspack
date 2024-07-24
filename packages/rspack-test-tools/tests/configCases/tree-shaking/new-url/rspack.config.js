/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	mode: "development",
	context: __dirname,
	module: {
		rules: [
			{
				test: /\.svg$/,
				type: "asset/resource"
			}
		]
	},

	optimization: {
		sideEffects: true
	},
	output: {
		chunkFilename: "[name].js"
	},
	externalsPresets: {
		node: true
	}
};
