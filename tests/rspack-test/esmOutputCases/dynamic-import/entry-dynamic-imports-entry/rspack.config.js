module.exports = {
  entry: {
    main: './entry-a.js',
    b: './entry-b.js',
  },
  optimization: {
    runtimeChunk: false,
    splitChunks: false,
  },
};
