let firstChunkAsset = null;

class CheckAssetPlugin {
	apply(compiler) {
		compiler.hooks.thisCompilation.tap("TestPlugin", compilation => {
			compilation.hooks.afterProcessAssets.tap("TestPlugin", assets => {
				const chunkAsset = Object.keys(assets).find(asset =>
					asset.startsWith("chunk.")
				);
				if (!chunkAsset) {
					throw new Error("chunk asset not found");
				}
				if (firstChunkAsset === null) {
					firstChunkAsset = chunkAsset;
				} else {
					expect(firstChunkAsset).not.toEqual(chunkAsset);
				}
			});
		});
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = [
	{
		mode: "production",
		entry: "./main.js",
		output: {
			chunkFilename: "chunk.[chunkhash].js"
		},
		optimization: {
			concatenateModules: true,
			minimize: false
		},
		plugins: [new CheckAssetPlugin()]
	},
	{
		mode: "production",
		entry: "./main.js",
		output: {
			chunkFilename: "chunk.[chunkhash].js"
		},
		optimization: {
			concatenateModules: true,
			minimize: false
		},
		plugins: [new CheckAssetPlugin()]
	}
];
