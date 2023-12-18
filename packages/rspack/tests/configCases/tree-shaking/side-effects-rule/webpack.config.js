/**@type {import('@rspack/cli').Configuration}*/
module.exports = {
	context: __dirname,
	mode: "production",
	module: {
		rules: [
			{
				test: /package/,
				sideEffects: false
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
