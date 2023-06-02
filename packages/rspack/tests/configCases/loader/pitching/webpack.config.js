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
					"./simple-loader.js",
					"./pitching-loader.js",
					"./simple-async-loader.js"
				]
			}
		]
	}
};
