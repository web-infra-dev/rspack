/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	module: {
		rules: [
			{
				test: /side-effect\.js/,
				sideEffects: false,
			}
		]
	},
	optimization: {
		sideEffects: false,
	},
};
