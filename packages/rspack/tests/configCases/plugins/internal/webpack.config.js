const { RawSource, ConcatSource } = require("webpack-sources");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	plugins: [
		{
			name: "test",
			apply(compiler) {
				compiler.hooks.compilation.tap("compilation", compilation => {
					compilation.hooks.processAssets.tapPromise(
						"processAssets1",
						async assets => {
							for (const [key, value] of Object.entries(assets)) {
								compilation.updateAsset(
									key,
									new ConcatSource(new RawSource("//banner;\n"), value)
								);
							}
						}
					);
				});
			}
		}
	]
};
