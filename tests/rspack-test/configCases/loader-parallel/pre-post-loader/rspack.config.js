/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	module: {
		rules: [
			{
				test: /a\.js$/,
				use: { loader: "./loader1", parallel: { maxWorkers: 4 }, options: {} }
			},
			{
				test: /a\.js$/,
				use: { loader: "./loader2", parallel: { maxWorkers: 4 }, options: {} },
				enforce: "pre"
			},
			{
				test: /a\.js$/,
				use: {
					loader: "./loader3",
					parallel: { maxWorkers: 4 },
					options: {}
				},
				enforce: "post"
			}
		]
	},
	experiments: {
		parallelLoader: true
	}
};
