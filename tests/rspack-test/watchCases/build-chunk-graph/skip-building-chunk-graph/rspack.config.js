/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		sideEffects: true,
		usedExports: false,
		innerGraph: true
	},
	module: {
		rules: [
			{
				test: /re-exports\.js$/,
				sideEffects: false
			}
		]
	}
};
