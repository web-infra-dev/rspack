const rspack = require('@rspack/core');

const checkChunkIds = (minLength) => (compiler) => {
  compiler.hooks.done.tap('CheckCompactChunkIds', (stats) => {
    const chunks = stats.toJson({
      all: false,
      chunks: true,
      ids: true,
    }).chunks;
    for (const chunk of chunks) {
      expect(chunk.id).toMatch(/^[A-Za-z][A-Za-z0-9]*$/);
      expect(chunk.id.length).toBeGreaterThanOrEqual(minLength);
    }
  });
};

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
  {
    optimization: {
      chunkIds: 'compact',
    },
    plugins: [checkChunkIds(1)],
  },
  {
    optimization: {
      chunkIds: 'natural',
    },
    plugins: [
      new rspack.ids.CompactChunkIdsPlugin({ minLength: 2 }),
      checkChunkIds(2),
    ],
  },
];
