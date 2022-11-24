/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: /\.js/,
				use: [
					{
						loader: "./my-loader.js"
					}
				]
			}
		]
	}
};
