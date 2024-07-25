/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: 'web',
	module: {
		generator: {
			'css/auto': {
				exportsConvention: 'camel-case-only',
				localIdentName: '[local]',
				exportsOnly: false,
			}
		}
	},
	mode: "development",
	experiments: {
		css: true
	}
};
