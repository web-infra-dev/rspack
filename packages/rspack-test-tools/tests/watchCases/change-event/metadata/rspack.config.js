const path = require("node:path");
const fs = require("node:fs");

class ShouldRebuildPlugin {
	constructor() {
		this.assetName = "bundle.js";
		this.watchingChangeCount = 0;
		this.compileCount = 0;
	}
	apply(compiler) {
		const targetFile = path.resolve(compiler.context, "./index.js");

		compiler.hooks.thisCompilation.tap(
			ShouldRebuildPlugin.name,
			compilation => {
				if (compiler.watching) {
					compiler.watching.onChange = () => {
						this.watchingChangeCount++;
					};
				} else {
					console.error("Compiler is not in watch mode");
				}
				const { Compilation } = compiler.rspack;
				compilation.hooks.processAssets.tap(
					{
						name: ShouldRebuildPlugin.name,
						stage: Compilation.PROCESS_ASSETS_STAGE_ADDITIONS
					},
					assets => {
						const asset = assets[this.assetName];
						if (!asset) {
							return;
						}

						const { RawSource } = compiler.rspack.sources;
						const oldContent = asset.source();
						const newContent = oldContent.replace(
							"GLOBAL_WATCH_CHANGE_COUNT",
							this.watchingChangeCount
						);
						const source = new RawSource(newContent);

						compilation.updateAsset(this.assetName, source);
					}
				);
			}
		);

		compiler.hooks.done.tap(ShouldRebuildPlugin.name, _ => {
			// After first compilation, touch the file to trigger a rebuild
			if (this.compileCount === 0) {
				fs.utimes(targetFile, Date.now(), Date.now(), err => {
					if (err) {
						console.error("Error updating file timestamps:", err);
						return;
					}
					// Touch file to trigger rebuild
				});
			}
			this.compileCount++;
		});
	}
}

/**
 * @type {import('@rspack/core').Configuration}
 */
const config = {
	cache: false,
	plugins: [new ShouldRebuildPlugin()]
};

module.exports = config;
