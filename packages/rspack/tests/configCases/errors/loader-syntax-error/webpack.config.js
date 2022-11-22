/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: "lib.js",
				uses: [
					{
						loader: "./my-loader.js"
					}
				]
			}
		]
	}
};
