/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /\.svg$/,
				resourceQuery: /inline/,
				type: "asset/inline"
			}
		]
	}
};
