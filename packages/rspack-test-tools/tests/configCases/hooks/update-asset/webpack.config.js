/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		filename: "[name].[contenthash].js"
	},
	plugins: [
		function plugin(compiler) {
			compiler.hooks.compilation.tap("test", (compilation) => {
				compilation.hooks.processAssets.tap({
					name: "test",
					stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS,
				}, (assets) => {
					Object.entries(assets).forEach(([filename, asset]) => {
						const newContent = `// UPDATED\n${asset.source()}`;
						compilation.updateAsset(filename, new compiler.webpack.sources.RawSource(newContent))
					})
				})
				compilation.hooks.processAssets.tap({
					name: "test",
					stage: compiler.webpack.Compilation.PROCESS_ASSETS_STAGE_OPTIMIZE_HASH,
				}, (assets) => {
					compilation.getAssets().forEach(({ info }) => {
						expect(info.contentHash.length).toBeGreaterThan(0)
					})
				})
			})
		}
	]
};
