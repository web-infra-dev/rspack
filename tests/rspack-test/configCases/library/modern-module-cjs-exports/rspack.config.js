module.exports = {
  mode: 'production',
  target: 'node22',
  cache: false,
  experiments: {
    outputModule: true,
  },
  module: {
    rules: [
      { test: /declaration-(object|array)\.js$/, type: 'javascript/auto' },
    ],
  },
  output: {
    filename: 'bundle0.mjs',
    library: { type: 'modern-module' },
  },
  optimization: { minimize: false, runtimeChunk: false },
};
