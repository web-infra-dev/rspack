// @ts-check

/** @type {import("@rspack/core").Configuration} */
// empty object is a valid configuration
export const shouldAcceptEmptyObject = {};

/** @type {import("@rspack/core").Configuration} */
export const shouldAcceptEmptyArr = [];

/** @type {import("@rspack/core").Configuration} */
export const shouldAcceptArrWithRealWorldConfig = [
	{
		context: __dirname,
		module: {
			rules: [
				{
					test: /\.svg$/,
					type: "asset/resource"
				}
			]
		},
		builtins: {
			treeShaking: true
		},
		optimization: {
			sideEffects: true
		},
		externalsPresets: {
			node: true
		}
	}
];

/** @type {import("@rspack/core").Configuration} */
export const shouldRejectArrWithWrongConfig = [
	{
		context: __dirname,
		module: {
			rules: [
				{
					// @ts-expect-error should be a regexp
					test: 1,
					type: "asset/resource"
				}
			]
		},
		builtins: {
			// @ts-expect-error should be boolean
			treeShaking: "1"
		},
		// @ts-expect-error should not be an array
		optimization: [],
		externalsPresets: {
			// @ts-expect-error should be string
			node: 1
		}
	}
];

module.exports = shouldAcceptArrWithRealWorldConfig;
