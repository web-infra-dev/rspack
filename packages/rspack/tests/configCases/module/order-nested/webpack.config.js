/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /lib.js/,
				rules: [
					{
						use: ["./loader2.js"]
					}
				],
				oneOf: [
					{
						resourceQuery: /random-string/,
						use: ["./loader1.js"]
					},
					{
						use: ["./loader.js"]
					}
				]
			}
		]
	}
};
