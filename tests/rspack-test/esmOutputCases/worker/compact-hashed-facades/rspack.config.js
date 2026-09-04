module.exports = ['compact-hashed', 'deterministic'].map((chunkIds) => ({
  name: chunkIds,
  output: {
    filename: `${chunkIds}/[name].mjs`,
    chunkFilename: `${chunkIds}/[name].mjs`,
  },
  optimization: {
    minimize: false,
    moduleIds: 'named',
    chunkIds,
    removeEmptyChunks: true,
    runtimeChunk: 'single',
    splitChunks: {
      cacheGroups: {
        workers: {
          test: /(worker|async)\.js/,
          name: 'workers',
          enforce: true,
        },
      },
    },
  },
}));
