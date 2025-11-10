/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /\.js$/,
				use: ["./loader.js"],
				oneOf: [
					{
						test: /lib\.js$/,
						use: ["./loader1.js"],
					},
					{
						test: /random-string/,
						use: ["./loader2.js"],
					},
				],
			}
		]
	}
};
