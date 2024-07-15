/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	output: {
		publicPath: "/",
		filename: 'main.js'
	},
	module: {
		rules: [
			{
				test: /\.js$/,
				type: "javascript/auto",
				generator: {
					filename: "custom/lib.js"
				}
			},
		]
	}
};
