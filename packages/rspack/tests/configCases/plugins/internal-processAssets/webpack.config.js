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
					compilation.hooks.processAssets.tapPromise("Test1", async assets => {
						for (const [key, value] of Object.entries(assets)) {
							compilation.updateAsset(
								key,
								new ConcatSource(new RawSource("//banner;\n"), value)
							);
						}
					});

					compilation.hooks.processAssets.tapPromise("Test2", async assets => {
						assert((Object.keys(assets).length = 1));
						assert((Object.getOwnPropertyNames(assets).length = 1));
						assert((Reflect.ownKeys(assets).length = 1));
						assert("main.js" in assets);
						assert(assets["main.js"].source().startsWith("//banner;\n"));
					});

					compilation.hooks.processAssets.tapAsync(
						{
							name: "Test3",
							stage:
								compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONAL
						},
						async (assets, callback) => {
							for (const [key, value] of Object.entries(assets)) {
								compilation.updateAsset(
									key,
									new ConcatSource(
										new RawSource("//stage_additional;\n"),
										value
									)
								);
							}
							callback();
						}
					);

					compilation.hooks.processAssets.tapAsync(
						{
							name: "Test4",
							stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_SUMMARIZE
						},
						async (assets, callback) => {
							for (const [key, value] of Object.entries(assets)) {
								value.source().startsWith("//banner;\n//stage_additional;\n");
							}
							callback();
						}
					);
				});
			}
		}
	]
};
