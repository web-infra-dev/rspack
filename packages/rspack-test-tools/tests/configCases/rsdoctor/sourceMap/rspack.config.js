const {
	experiments: { RsdoctorPlugin }
} = require("@rspack/core");
const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	devtool: false,
	plugins: [
		new RsdoctorPlugin({
			moduleGraphFeatures: false,
			chunkGraphFeatures: false,
			sourceMapFeatures: {
				cheap: true,
				module: true
			}
		}),
		{
			apply(compiler) {
				compiler.hooks.afterEmit.tap("TestPlugin::SourceMap", compilation => {
					const assets = compilation.getAssets();

					console.log(
						"Generated assets:",
						assets.map(a => a.name)
					);

					// Check if each JS and CSS asset has a source map
					const jsCssAssets = assets.filter(
						asset => asset.name.endsWith(".js") || asset.name.endsWith(".css")
					);

					jsCssAssets.forEach(asset => {
						console.log(
							`Asset: ${asset.name}, has source map: ${asset.source.map() !== null}`
						);
						expect(asset.source.map()).toBeDefined();
						expect(asset.source.map()).not.toBeNull();
					});

					const sourceMapAssets = assets.filter(asset =>
						asset.name.endsWith(".map")
					);
					expect(sourceMapAssets.length).toBe(0);
				});
			}
		},
		{
			apply(compiler) {
				compiler.hooks.afterEmit.tap("TestPlugin::CheapOnly", compilation => {
					const assets = compilation.getAssets();
					const jsCssAssets = assets.filter(
						asset => asset.name.endsWith(".js") || asset.name.endsWith(".css")
					);
					// Check if each asset has a source map
					jsCssAssets.forEach(asset => {
						console.log(
							`Asset: ${asset.name}, has source map: ${asset.source.map() !== null}`
						);
						expect(asset.source.map()).toBeDefined();
						expect(asset.source.map()).not.toBeNull();
					});

					const sourceMapAssets = assets.filter(asset =>
						asset.name.endsWith(".map")
					);
					expect(sourceMapAssets.length).toBe(0);
				});
			}
		}
	]
};
