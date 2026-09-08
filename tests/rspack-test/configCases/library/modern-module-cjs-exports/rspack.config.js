module.exports = {
  mode: 'production',
  target: 'node22',
  cache: false,
  experiments: {
    outputModule: true,
  },
  output: {
    filename: 'bundle0.mjs',
    library: { type: 'modern-module' },
  },
  optimization: { minimize: false, runtimeChunk: false },
};
