/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	module: {
		rules: [
			{
				test: /a\.js$/,
				use: { loader: "./loader1", parallel: true, options: {} }
			},
			{
				test: /a\.js$/,
				use: { loader: "./loader2", parallel: true, options: {} },
				enforce: "pre"
			},
			{
				test: /a\.js$/,
				use: {
					loader: "./loader3",
					parallel: true,
					options: {}
				},
				enforce: "post"
			}
		]
	},
};
