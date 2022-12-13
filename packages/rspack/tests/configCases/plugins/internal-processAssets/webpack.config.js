const assert = require("assert");
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

					compilation.hooks.processAssets.tapPromise(
						"processAssets2",
						async assets => {
							assert((Object.keys(assets).length = 1));
							assert((Object.getOwnPropertyNames(assets).length = 1));
							assert((Reflect.ownKeys(assets).length = 1));
							assert("main.js" in assets);
							assert(assets["main.js"].source().startsWith("//banner;\n"));
						}
					);
				});
			}
		}
	]
};
