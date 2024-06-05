class MyPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("Plugin", compilation => {
			compilation.hooks.processAssets.tap(
				{ name: "Plugin", stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_SIZE - 1 },
				() => compilation
					.getAssets()
					.forEach(a => {
						const new_info = {...a.info, unminified_name: a.name ?? "non_empty_str"};
						compilation.updateAsset(a.name, a.source, new_info);
					})
			);

			compiler.hooks.done.tapPromise("Plugin", async (stats) => {
				stats.compilation
					.getAssets()
					.forEach(a => {
						expect(a.info.unminified_name).toBeTruthy();
					})
			})
		});
	}
}

/** @type {import('../..').TCompilerCaseConfig} */
module.exports = {
	description: "compilation.updateAsset should preserve fields in addition to KnownAssetInfo",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [new MyPlugin()]
		};
	},
};
