const PLUGIN_NAME = "MyPlugin";

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	cache: true,
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.compilation.tap(PLUGIN_NAME, compilation => {
					compilation.hooks.finishModules.tapPromise(PLUGIN_NAME, async () => {
						const cache = compilation.getCache(PLUGIN_NAME);
						await cache.storePromise("data", null, "some data");
					});
					compilation.hooks.processAssets.tapPromise(PLUGIN_NAME, async () => {
						const cache = compilation.getCache(PLUGIN_NAME);
						const data = await cache.getPromise("data", null);
						expect(data).toBe("some data");
					});
				});
			}
		}
	]
};
