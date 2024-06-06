/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		sideEffects: true
	},
	module: {
		parser: {
			'css/auto': {
				namedExports: false
			}
		}
	},
	experiments: {
		css: true
	}
};
