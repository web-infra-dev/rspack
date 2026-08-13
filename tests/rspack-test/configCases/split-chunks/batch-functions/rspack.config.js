const callbackStats = {
  layer: { batches: 0, calls: 0, active: false },
  test: { batches: 0, calls: 0, active: false },
  chunks: { batches: 0, calls: 0, active: false },
  name: { batches: 0, calls: 0, active: false },
};
const nameChunkArrays = new Set();
const nameModuleOrder = [];

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
        calls: 7,
        active: false,
      });
      expect(callbackStats.test).toEqual({
        batches: 1,
        calls: 7,
        active: false,
      });
      expect(callbackStats.chunks.calls).toBe(16);
      expect(callbackStats.chunks.batches).toBeLessThan(callbackStats.chunks.calls);
      expect(callbackStats.chunks.active).toBe(false);
      expect(callbackStats.name.calls).toBe(6);
      expect(callbackStats.name.batches).toBeLessThan(callbackStats.name.calls);
      expect(callbackStats.name.active).toBe(false);
      expect(nameChunkArrays.size).toBe(6);
      expect(new Set(nameModuleOrder.slice(0, 3)).size).toBe(3);
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
    c: {
      import: './c',
      layer: 'batch',
    },
    d: {
      import: './d',
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
            return /shared-[123]\.js/.test(module.identifier());
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
            expect(['a,b', 'a,b,c', 'a,b,c,d']).toContain(
              chunks.map((chunk) => chunk.name).sort().join(','),
            );
            expect(cacheGroupKey).toBe('batch');
            nameChunkArrays.add(chunks);
            nameModuleOrder.push(module.identifier());
            return 'shared';
          },
        },
      },
    },
  },
  plugins: [new AssertBatchCallbacksPlugin()],
};
