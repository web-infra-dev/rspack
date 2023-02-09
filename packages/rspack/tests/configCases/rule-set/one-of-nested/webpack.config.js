const path = require("path");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /lib.js/,
				use: [
					{
						loader: "./loader2.js"
					}
				]
			},
			{
				test: /lib.js/,
				oneOf: [
					{
						oneOf: [
							{
								resourceQuery: "/(__inline=false|url)/",
								use: [
									{
										loader: "./loader1.js"
									}
								]
							},
							{
								use: [
									{
										loader: "./loader.js"
									}
								]
							}
						]
					},
					{
						use: [
							{
								loader: "./loader1.js"
							}
						]
					}
				]
			}
		]
	}
};
