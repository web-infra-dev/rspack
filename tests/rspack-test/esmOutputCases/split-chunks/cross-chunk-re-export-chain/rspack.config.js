module.exports = {
  optimization: {
    splitChunks: {
      cacheGroups: {
        middle: {
          test: /middle\.js$/,
          name(module, chunks, cacheGroupKey) {
            expect(Array.isArray(module)).toBe(false);
            expect(typeof module.identifier).toBe('function');
            expect(Array.isArray(chunks)).toBe(true);
            for (const chunk of chunks) void chunk.name;
            expect(cacheGroupKey).toBe('middle');
            return 'middle-chunk';
          },
        },
        leaf: {
          test: /leaf\.js$/,
          name(module, chunks, cacheGroupKey) {
            expect(Array.isArray(module)).toBe(false);
            expect(typeof module.identifier).toBe('function');
            expect(Array.isArray(chunks)).toBe(true);
            for (const chunk of chunks) void chunk.name;
            expect(cacheGroupKey).toBe('leaf');
            return 'leaf-chunk';
          },
        },
      },
    },
  },
};
