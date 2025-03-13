/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	module: {
		rules: [
			{
				test: /a\.js$/,
				use: "./loader1",
				parallel: true,
				options: {}
			},
			{
				test: /a\.js$/,
				use: "./loader2",
				enforce: "pre",
				parallel: true,
				options: {}
			},
			{
				test: /a\.js$/,
				use: "./loader3",
				enforce: "post",
				parallel: true,
				options: {}
			}
		]
	},
	experiments: {
		parallelLoader: true
	}
};
