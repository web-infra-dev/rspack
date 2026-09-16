/** @type {import('@rspack/core').Configuration} */
export default {
  entry: {
    'entry?query': './index.js',
  },
  output: {
    filename: '[contenthash].js',
    chunkFilename: () => '[name].js',
  },
};
