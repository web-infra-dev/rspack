module.exports = ['development', 'production'].map((mode) => ({
  mode,
  target: 'web',
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  node: { __dirname: false, __filename: false },
  module: {
    rules: [{ test: /\.css$/, type: 'css/module' }],
  },
  optimization: { minimize: false },
  experiments: { css: true },
}));
