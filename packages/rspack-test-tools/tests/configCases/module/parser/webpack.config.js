/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		parser: {
			asset: {
				dataUrlCondition: {
					// Size of the `logo.png` is 700+ bytes
					maxSize: 1000
				}
			}
		},
		rules: [
			{
				test: /\.png$/,
				resourceQuery: /should-be-externalized/,
				type: "asset",
				parser: {
					dataUrlCondition: {
						maxSize: 100
					}
				}
			},
			{
				test: /\.png$/,
				resourceQuery: /should-be-inlined/,
				type: "asset"
			}
		]
	}
};
