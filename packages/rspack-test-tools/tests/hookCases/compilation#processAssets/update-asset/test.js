let contentHashes = [];
let files = [];

/** @type {import("../../../..").THookCaseConfig} */
module.exports = {
	description: "should emit assets correctly",
	findBundle() {
		return files;
	},

	options(context) {
		return {
			output: {
				filename: '[name].[contenthash].js'
			},
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

						compilation.hooks.afterProcessAssets.tap(
							{
								name: "test",
								stage:
									compiler.webpack.Compilation
										.PROCESS_ASSETS_STAGE_OPTIMIZE_HASH
							},
							(assets) => {
								files.push(...Object.keys(assets));
							}
						);
					});
				}
			]
		};
	},
	async check() {
		contentHashes.forEach(hash => {
			expect(hash.length).toBeGreaterThan(0);
		});
	}
};
