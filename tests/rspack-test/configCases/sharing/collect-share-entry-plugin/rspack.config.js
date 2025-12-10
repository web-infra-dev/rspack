
const { container,sources } = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new container.ModuleFederationPlugin({
			shared: {
				xreact: {
					import: "xreact",
					shareKey: "xreact",
					shareScope: "default",
					version: "1.0.0",
					treeshake: true
				}
			},
		}),
		{
			apply(compiler) {
				compiler.hooks.thisCompilation.tap("CollectSharedEntryPlugin", (compilation) => {
					compilation.hooks.processAssets.tapPromise(
						{
							name: "emitCollectSharedEntry",
							stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE
						},
						async () => {
							const filename = 'collect-shared-entries.json';
							const asset = compilation.getAsset(filename);
							if (!asset) {
								return;
							}
							compilation.emitAsset('collect-shared-entries-copy.json', new sources.RawSource(asset.source.source().toString()));
						}
					);
				});
			}
		}
	]
};
