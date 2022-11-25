/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /lib\.js$/,
				use: [
					{
						loader: "./my-loader.js"
					}
				]
			}
		]
	}
};
