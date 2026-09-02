const assert = require('assert');

const PLUGIN_NAME = 'after-optimize-chunk-ids-test-plugin';

class Plugin {
  apply(compiler) {
    let hookCalled = false;
    let idsFromHook = [];
    let processAssetsCalled = false;

    compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
      assert(typeof compilation.hooks.afterOptimizeChunkIds !== 'undefined');

      compilation.hooks.afterOptimizeChunkIds.tap(PLUGIN_NAME, (chunks) => {
        hookCalled = true;
        // chunk ids are already assigned when this hook runs
        assert(
          !processAssetsCalled,
          'afterOptimizeChunkIds should run before processAssets',
        );
        idsFromHook = [...chunks].map((chunk) => {
          assert(
            typeof chunk.id === 'string' || typeof chunk.id === 'number',
            `chunk id should be assigned, got ${chunk.id}`,
          );
          return chunk.id;
        });
      });

      compilation.hooks.processAssets.tap(PLUGIN_NAME, () => {
        processAssetsCalled = true;
      });
    });

    compiler.hooks.done.tap(PLUGIN_NAME, (stats) => {
      assert(hookCalled, 'afterOptimizeChunkIds should be called');
      // one initial chunk plus one async chunk
      assert(
        idsFromHook.length === 2,
        `expected 2 chunks, got ${idsFromHook.length}`,
      );
      const idsFromStats = stats
        .toJson({ chunks: true })
        .chunks.map((chunk) => chunk.id);
      assert.deepStrictEqual(idsFromHook.slice().sort(), idsFromStats.sort());
    });
  }
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
  context: __dirname,
  plugins: [new Plugin()],
};
