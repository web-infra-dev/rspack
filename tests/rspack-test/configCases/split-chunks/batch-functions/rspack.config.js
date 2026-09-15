const callbackStats = {
  batches: 0,
  calls: 0,
  active: false,
};
const nameChunkArrays = new Set();
const nameModuleOrder = [];

function trackCallback() {
  callbackStats.calls++;
  if (!callbackStats.active) {
    callbackStats.active = true;
    callbackStats.batches++;
    queueMicrotask(() => {
      callbackStats.active = false;
    });
  }
}

class AssertBatchCallbacksPlugin {
  apply(compiler) {
    compiler.hooks.done.tap('AssertBatchCallbacksPlugin', () => {
      expect(callbackStats.calls).toBe(6);
      expect(callbackStats.batches).toBeLessThan(callbackStats.calls);
      expect(callbackStats.active).toBe(false);
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
          layer: 'batch',
          test: /shared-[123]\.js/,
          chunks: 'all',
          name(module, chunks, cacheGroupKey) {
            trackCallback();
            expect(Array.isArray(module)).toBe(false);
            expect(Array.isArray(chunks)).toBe(true);
            expect(['a,b', 'a,b,c', 'a,b,c,d']).toContain(
              chunks
                .map((chunk) => chunk.name)
                .sort()
                .join(','),
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
