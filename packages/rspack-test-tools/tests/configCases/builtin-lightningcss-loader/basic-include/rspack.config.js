const rspack = require('@rspack/core')
const browserslist = require('browserslist')

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	module: {
		parser: {
			'css/auto': {
				namedExports: true
			}
		},
		rules: [
			{
				test: /\.css$/,
				use: [
					{
						loader: "builtin:lightningcss-loader",
						/** @type {import("@rspack/core").LightningcssLoaderOptions} */
						options: {
							unusedSymbols: ['unused'],
							targets: 'ie 10',
							include: {
								nesting: true
							}
						}
					}
				],
				type: "css/auto"
			}
		]
	},
	experiments: {
		css: true
	}
};
