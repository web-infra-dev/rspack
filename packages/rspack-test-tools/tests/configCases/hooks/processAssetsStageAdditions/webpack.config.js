const { RawSource } = require("webpack-sources");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		new (class {
			constructor(banner) {
				this.banner = banner;
				this.name = "BannerPlugin";
			}
			apply(compiler) {
				const banner = this.banner;
				compiler.hooks.compilation.tap("BannerPlugin", compilation => {
					compilation.hooks.processAssets.tap(
						{
							name: "BannerPlugin",
							// ProcessAssetsStageAdditions
							stage: -100
						},
						assets => {
							for (const file of Object.keys(assets)) {
								compilation.updateAsset(file, old => {
									const newContent = `${banner}\n${old.source().toString()}`;
									return new RawSource(newContent);
								});
							}
						}
					);
				});
			}
		})("/** MMMMM */")
	]
};
