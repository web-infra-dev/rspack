const assert = require("assert");
const { RawSource, ConcatSource } = require("webpack-sources");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	plugins: [
		{
			name: "test",
			apply(compiler) {
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
										new RawSource("//PROCESS_ASSETS_STAGE_ADDITIONAL;\n"),
										value
									)
								);
							}
						}
					);
					compilation.hooks.processAssets.tapPromise(
						{
							name: "Test",
							stage:
								compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_PRE_PROCESS
						},
						async assets => {
							for (const [key, value] of Object.entries(assets)) {
								compilation.updateAsset(
									key,
									new ConcatSource(
										new RawSource("//PROCESS_ASSETS_STAGE_PRE_PROCESS;\n"),
										value
									)
								);
							}
						}
					);
					compilation.hooks.processAssets.tapPromise("Test", async assets => {
						for (const [key, value] of Object.entries(assets)) {
							compilation.updateAsset(
								key,
								new ConcatSource(
									new RawSource("//PROCESS_ASSETS_STAGE_NONE;\n"),
									value
								)
							);
						}
					});
					compilation.hooks.processAssets.tapPromise(
						{
							name: "Test",
							stage:
								compiler.webpack.Compilation
									.PROCESS_ASSETS_STAGE_OPTIMIZE_INLINE
						},
						async assets => {
							for (const [key, value] of Object.entries(assets)) {
								compilation.updateAsset(
									key,
									new ConcatSource(
										new RawSource("//PROCESS_ASSETS_STAGE_OPTIMIZE_INLINE;\n"),
										value
									)
								);
							}
						}
					);
					compilation.hooks.processAssets.tapPromise(
						{
							name: "Test",
							stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_SUMMARIZE
						},
						async assets => {
							for (const [key, value] of Object.entries(assets)) {
								compilation.updateAsset(
									key,
									new ConcatSource(
										new RawSource("//PROCESS_ASSETS_STAGE_SUMMARIZE;\n"),
										value
									)
								);
							}
						}
					);
					compilation.hooks.processAssets.tapPromise(
						{
							name: "Test",
							stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_REPORT
						},
						async assets => {
							for (const [key, value] of Object.entries(assets)) {
								compilation.updateAsset(
									key,
									new ConcatSource(
										new RawSource("//PROCESS_ASSETS_STAGE_REPORT;\n"),
										value
									)
								);
							}
						}
					);
				});
			}
		}
	]
};
