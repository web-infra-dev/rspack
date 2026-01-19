/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /\.svg$/,
				type: "asset/resource"
			},
			{
				test: /\.css/,
				type: 'css/auto'
			}
		]
	},
	optimization: {
		sideEffects: true
	},
	externalsPresets: {
		node: true
	}
};
