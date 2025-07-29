const {
	experiments: { RsdoctorPlugin }
} = require("@rspack/core");
const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	devtool: "source-map",
	plugins: [
		new RsdoctorPlugin({
			moduleGraphFeatures: false,
			chunkGraphFeatures: false,
			sourceMapFeatures: {
				cheap: true
			}
		}),
		{
			apply(compiler) {
				compiler.hooks.afterEmit.tap("TestPlugin::SourceMap", compilation => {
					const assets = compilation.getAssets();
					const sourceMapAssets = assets.filter(asset =>
						asset.name.endsWith(".map")
					);

					console.log(
						"Generated assets:",
						assets.map(a => a.name)
					);
					console.log(
						"Source map assets:",
						sourceMapAssets.map(a => a.name)
					);

					expect(sourceMapAssets.length).toBeGreaterThan(0);
					const mainSourceMap = sourceMapAssets.find(asset =>
						asset.name.includes("bundle0.js.map")
					);
					expect(mainSourceMap).toBeDefined();
					expect(mainSourceMap.source.size()).toBeGreaterThan(0);
					const sourceMapContent = mainSourceMap.source.source();
					expect(sourceMapContent).toContain('"sources"');
					expect(sourceMapContent).toContain('"mappings"');
				});
			}
		},
		{
			apply(compiler) {
				compiler.hooks.afterEmit.tap("TestPlugin::CheapOnly", compilation => {
					const assets = compilation.getAssets();
					const sourceMapAssets = assets.filter(asset =>
						asset.name.endsWith(".map")
					);
					expect(sourceMapAssets.length).toBeGreaterThan(0);
				});
			}
		}
	]
};
