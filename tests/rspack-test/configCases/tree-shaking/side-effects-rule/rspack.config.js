/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /package/,
				sideEffects: false
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
