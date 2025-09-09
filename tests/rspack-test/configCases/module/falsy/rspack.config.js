/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			undefined,
			{
				test: /lib.js/,
				use: ["./loader2.js"]
			},
			{
				test: /lib.js/,
				oneOf: [
					undefined,
					{
						resourceQuery: "/(__inline=false|url)/",
						use: ["./loader1.js"]
					},
					{
						use: ["./loader.js"]
					},
					{
						use: ["./loader1.js"]
					}
				]
			}
		]
	}
};
