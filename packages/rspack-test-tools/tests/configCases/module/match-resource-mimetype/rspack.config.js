const path = require("path");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	module: {
		rules: [
			{
				include: path.resolve(__dirname, 'a.js'),
				use: [
					'./get-source.js',
					{
						loader: "builtin:swc-loader",
						options: {
							jsc: {
								target: "es3",
							}
						}
					}
				]
			},
		]
	}
};
