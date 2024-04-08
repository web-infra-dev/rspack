/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	module: {
		generator: {
			"css": {
				exportsOnly: true,
			}
		},
		rules: [
			{
				test: /\.module\.css$/,
				type: "css/module",
				generator: {
					localIdentName: "[path][name][ext]-[local]",
					exportsOnly: false,
				}
			},
			{
				test: /\.css$/,
				type: "css",
			},
		]
	}
};
