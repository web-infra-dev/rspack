/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				use: ["./loader.js"],
				rules: [
					{
						test: /\.js$/,
						use: ["./loader1.js"],
					},
				],
				oneOf: [
					{
						test: /lib\.js$/,
						use: ["./loader2.js"],
					},
					{
						test: /random-string/,
						use: ["./loader3.js"],
					},
				],
			}
		]
	}
};
