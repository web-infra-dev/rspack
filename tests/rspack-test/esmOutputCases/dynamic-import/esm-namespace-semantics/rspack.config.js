module.exports = {
  output: {
    filename: 'main.mjs',
    chunkFilename: '[name].mjs',
  },
  optimization: {
    runtimeChunk: false,
  },
};
