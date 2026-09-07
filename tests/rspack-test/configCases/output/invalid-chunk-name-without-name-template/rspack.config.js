/** @type {import('@rspack/core').Configuration} */
module.exports = {
  entry: {
    'entry?query': './index.js',
  },
  output: {
    filename: '[contenthash].js',
    chunkFilename: () => '[name].js',
  },
};
