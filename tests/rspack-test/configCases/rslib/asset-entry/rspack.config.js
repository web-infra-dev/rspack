const rspack = require('@rspack/core');

module.exports = {
	entry: './index.png',
	module: {
		rules: [
			{
				test: /\.png$/,
				type: 'asset/resource'
			}
		]
	},
	plugins: [
		new rspack.experiments.RslibPlugin(),
		(
			/**@type {import('@rspack/core').Compiler} */ compiler
		) => {
			compiler.hooks.done.tap('test case', stats => {
				const asset = stats.compilation.getAsset('bundle0.js');
				expect(asset).toBeDefined();
			})
		}
	],
}
