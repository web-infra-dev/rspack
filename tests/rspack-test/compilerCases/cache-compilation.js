const mockFn = rstest.fn();
const PLUGIN_NAME = "MyPlugin";

class MyPlugin {
  apply(compiler) {
    compiler.hooks.compilation.tap(PLUGIN_NAME, compilation => {
      compilation.hooks.processAssets.tapPromise(PLUGIN_NAME, async () => {
        const cache = compilation.getCache(PLUGIN_NAME);
        const currentAssets = compilation.getAssets().map(i => i.name);
        const lastAssets = await cache.getPromise('assets', null);
        if (lastAssets) {
          expect(currentAssets).toEqual(lastAssets);
        } else {
          await cache.storePromise('assets', null, currentAssets);
        }
        mockFn();
      });
    });
  }
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
  description: "should share cache cross compilations",
  options(context) {
    return {
      context: context.getSource(),
      entry: "./d",
      plugins: [new MyPlugin()]
    };
  },
  async build(_, compiler) {
    await new Promise(resolve => {
      compiler.run(() => {
        compiler.run(() => {
          resolve();
        });
      });
    });
  },
  async check() {
    expect(mockFn).toBeCalledTimes(2);
  }
};
