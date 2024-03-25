/** @type {import('@rspack/core').Configuration} */
module.exports = {
	entry: "./index",
	stats: "errors-warnings",
	optimization: {
		minimize: true,
	},
	plugins: [
		{
			apply(compiler) {
				const { ConcatSource, RawSource } = compiler.webpack.sources;
				compiler.hooks.compilation.tap("compilation", compilation => {
					compilation.hooks.processAssets.tapPromise(
						{
							name: "Test",
							stage:
								compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL
						},
						async assets => {
							for (const [key, value] of Object.entries(assets)) {
								compilation.updateAsset(
									key,
									new ConcatSource(
										new RawSource("const a {}\n"),
										value
									)
								);
							}
						}
					);
				});
			}
		},
	],
}
