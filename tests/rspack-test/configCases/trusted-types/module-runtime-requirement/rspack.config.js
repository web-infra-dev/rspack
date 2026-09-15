module.exports = {
  externals: {
    fs: 'node-commonjs fs',
    path: 'node-commonjs path',
  },
  output: {
    filename: '[name].js',
    chunkFilename: '[name].js',
    trustedTypes: true,
  },
  node: {
    __dirname: false,
    __filename: false,
  },
  devtool: 'eval-source-map',
  target: 'web',
};
