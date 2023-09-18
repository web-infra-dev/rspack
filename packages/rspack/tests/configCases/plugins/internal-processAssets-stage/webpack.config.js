const { RawSource, ConcatSource } = require("webpack-sources");

const NAME = "TestPlugin";

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	plugins: [
		{
			name: NAME,
			apply(compiler) {
				compiler.hooks.compilation.tap("compilation", compilation => {
					function addStage(stage) {
						compilation.hooks.processAssets.tapPromise(
							{
								name: NAME,
								stage: compiler.webpack.Compilation[stage]
							},
							async assets => {
								for (const [key, value] of Object.entries(assets)) {
									compilation.updateAsset(
										key,
										new ConcatSource(new RawSource(`//${stage};\n`), value)
									);
								}
							}
						);
					}

					addStage("PROCESS_ASSETS_STAGE_ADDITIONAL");
					addStage("PROCESS_ASSETS_STAGE_PRE_PROCESS");
					addStage("PROCESS_ASSETS_STAGE_DERIVED");
					addStage("PROCESS_ASSETS_STAGE_ADDITIONS");
					addStage("PROCESS_ASSETS_STAGE_NONE");
					addStage("PROCESS_ASSETS_STAGE_OPTIMIZE");
					addStage("PROCESS_ASSETS_STAGE_OPTIMIZE_COUNT");
					addStage("PROCESS_ASSETS_STAGE_OPTIMIZE_COMPATIBILITY");
					addStage("PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE");
					addStage("PROCESS_ASSETS_STAGE_DEV_TOOLING");
					addStage("PROCESS_ASSETS_STAGE_OPTIMIZE_INLINE");
					addStage("PROCESS_ASSETS_STAGE_SUMMARIZE");
					addStage("PROCESS_ASSETS_STAGE_OPTIMIZE_HASH");
					addStage("PROCESS_ASSETS_STAGE_OPTIMIZE_TRANSFER");
					addStage("PROCESS_ASSETS_STAGE_ANALYSE");
					addStage("PROCESS_ASSETS_STAGE_REPORT");
				});
			}
		}
	]
};
