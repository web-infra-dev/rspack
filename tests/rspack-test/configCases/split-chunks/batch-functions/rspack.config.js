const callbackStats = {
  layer: { batches: 0, calls: 0, active: false },
  test: { batches: 0, calls: 0, active: false },
  chunks: { batches: 0, calls: 0, active: false },
  name: { batches: 0, calls: 0, active: false },
};

function trackCallback(kind) {
  const stats = callbackStats[kind];
  stats.calls++;
  if (!stats.active) {
    stats.active = true;
    stats.batches++;
    queueMicrotask(() => {
      stats.active = false;
    });
  }
}

class AssertBatchCallbacksPlugin {
  apply(compiler) {
    compiler.hooks.done.tap('AssertBatchCallbacksPlugin', () => {
      expect(callbackStats.layer).toEqual({
        batches: 1,
        calls: 4,
        active: false,
      });
      expect(callbackStats.test).toEqual({
        batches: 1,
        calls: 4,
        active: false,
      });
      expect(callbackStats.chunks).toEqual({
        batches: 1,
        calls: 4,
        active: false,
      });
      expect(callbackStats.name).toEqual({
        batches: 1,
        calls: 2,
        active: false,
      });
    });
  }
}

/** @type {import('@rspack/core').Configuration} */
module.exports = {
  target: 'node',
  entry: {
    a: {
      import: './a',
      layer: 'batch',
    },
    b: {
      import: './b',
      layer: 'batch',
    },
  },
  output: {
    filename: '[name].js',
  },
  optimization: {
    concatenateModules: false,
    splitChunks: {
      chunks: 'all',
      minSize: 0,
      minChunks: 2,
      cacheGroups: {
        batch: {
          layer(layer) {
            trackCallback('layer');
            expect(typeof layer).toBe('string');
            return layer === 'batch';
          },
          test(module) {
            trackCallback('test');
            expect(Array.isArray(module)).toBe(false);
            return /shared-[12]\.js/.test(module.identifier());
          },
          chunks(chunk) {
            trackCallback('chunks');
            expect(Array.isArray(chunk)).toBe(false);
            return true;
          },
          name(module, chunks, cacheGroupKey) {
            trackCallback('name');
            expect(Array.isArray(module)).toBe(false);
            expect(Array.isArray(chunks)).toBe(true);
            expect(cacheGroupKey).toBe('batch');
            return 'shared';
          },
        },
      },
    },
  },
  plugins: [new AssertBatchCallbacksPlugin()],
};
