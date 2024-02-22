const rspack = require("@rspack/core");
/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
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
	},
	plugins: [
		new rspack.DefinePlugin({
			CONTEXT: JSON.stringify(__dirname)
		})
	]
};
