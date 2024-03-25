module.exports = class TestPlugin {
	constructor(cb) {
    this.cb = cb;
	}
	apply(compiler) {
    const list = []
    this.cb(compiler, list);
		compiler.hooks.compilation.tap(TestPlugin.name, compilation => {
			compilation.hooks.processAssets.tap(
				{
					name: TestPlugin.name,
					// ProcessAssetsStageAdditions
					stage: -100
				},
				assets => {
					for (const file of Object.keys(assets)) {
						compilation.updateAsset(file, old => {
							const newContent = `${list.join("\n")}\n${old.source().toString()}`;
							return new compiler.webpack.sources.RawSource(newContent);
						});
					}
				}
			);
		});
	}
}
