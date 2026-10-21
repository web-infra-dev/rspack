const assert = require('node:assert/strict');

module.exports = {
  devtool: 'source-map',
  optimization: { minimize: false },
  plugins: [
    {
      apply(compiler) {
        const snapshots = new WeakMap();
        const readAssets = (compilation) =>
          compilation.getAssets().map(({ name, source }) => ({
            name,
            content: source.buffer(),
            size: source.size(),
            map: source.map(),
          }));
        compiler.hooks.afterEmit.tap('ReadSourcesAfterEmit', (compilation) => {
          snapshots.set(compilation, readAssets(compilation));
        });
        compiler.hooks.done.tap('ReadSourcesAfterEmit', (stats) => {
          const expected = snapshots.get(stats.compilation);
          assert(expected.some((asset) => asset.name.endsWith('.map')));
          assert.deepEqual(readAssets(stats.compilation), expected);
        });
      },
    },
  ],
};
