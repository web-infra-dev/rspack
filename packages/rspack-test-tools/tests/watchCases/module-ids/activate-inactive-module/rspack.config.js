/** @type {import('@rspack/core').Configuration} */
module.exports = {
	module: {
		rules: [
			{
				test: /a\.js/,
				sideEffects: false
			}
		]
	}
};
