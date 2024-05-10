const { createFsFromVolume, Volume } = require("memfs");

const outputFileSystem = createFsFromVolume(new Volume());
let contentHashes = [];

module.exports = {
	description: "should emit assets correctly",
	options(context) {
		return {
			plugins: [
				function plugin(compiler) {
					compiler.hooks.compilation.tap("test", compilation => {
						compilation.hooks.processAssets.tap(
							{
								name: "test",
								stage:
									compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS
							},
							context.snapped(assets => {
								Object.entries(assets).forEach(([filename, asset]) => {
									const newContent = `// UPDATED\n${asset.source()}`;
									compilation.updateAsset(
										filename,
										new compiler.webpack.sources.RawSource(newContent)
									);
								});
							})
						);
						compilation.hooks.processAssets.tap(
							{
								name: "test",
								stage:
									compiler.webpack.Compilation
										.PROCESS_ASSETS_STAGE_OPTIMIZE_HASH
							},
							context.snapped(assets => {
								compilation.getAssets().forEach(({ info }) => {
									contentHashes.push(info.contentHash);
								});
							})
						);
					});
				}
			]
		};
	},
	async compiler(context, compiler) {
		compiler.outputFileSystem = outputFileSystem;
	},
	async check() {
		contentHashes.forEach(hash => {
			expect(hash.length).toBeGreaterThan(0);
		});
	}
};
