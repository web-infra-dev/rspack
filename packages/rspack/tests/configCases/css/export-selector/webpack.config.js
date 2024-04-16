/** @type {import("../../../../dist").Configuration} */
module.exports = {
	module: {
		generator: {
			'css/auto': {
				exportsConvention: 'camel-case-only',
				localIdentName: '[local]'
			}
		}
	},
	mode: "development",
	experiments: {
		css: true
	}
};
