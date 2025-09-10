class Plugin {
	/**
	 * @param {import('@rspack/core').Compiler} compiler
	 */
	apply(compiler) {
		compiler.hooks.afterEmit.tap("Test", compilation => {
			const assets = compilation.getAssets();
			for (const asset of assets) {
				const sourceMap = asset.info.related?.sourceMap;
				if (sourceMap) {
					expect(sourceMap).toBe(`${asset.name}.map`);
				}
			}
		});
	}
}

/**@type {import('@rspack/core').Configuration}*/
module.exports = {
	context: __dirname,
	output: {
		filename: "[name].[contenthash].js"
	},
	devtool: "source-map",
	plugins: [new Plugin()],
	module: {
		generator: {
			asset: {
				filename: "assets/[name].[contenthash][ext]"
			}
		},
		rules: [
			{
				test: /file\.txt$/,
				type: "asset/resource"
			}
		]
	}
};
