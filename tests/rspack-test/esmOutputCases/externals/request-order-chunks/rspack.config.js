module.exports = {
  entry: { main: './index.js', other: './other.js' },
  externals: {
    'order-a': 'module ./fixtures/a.mjs',
    'order-b': 'module ./fixtures/b.mjs',
    'order-c': 'module ./fixtures/c.mjs',
  },
  optimization: {
    runtimeChunk: false,
    splitChunks: {
      cacheGroups: {
        shared: {
          test: /shared\.js$/,
          name: 'shared',
          chunks: 'all',
          enforce: true,
        },
      },
    },
  },
};
