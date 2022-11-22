/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	context: __dirname,
	module: {
		rules: [
			{
				test: "\\.svg$",
				uses: [
					{
						loader: "./my-loader.js",
            type: "asset"
					}
				]
			}
		]
	}
};
