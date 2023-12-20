/**@type {import('@rspack/cli').Configuration}*/
module.exports = {
	mode: "production",
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
		minimize: false
	},
	externalsPresets: {
		node: true
	}
};
