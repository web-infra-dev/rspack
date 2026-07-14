'use strict';

module.exports = {
  entry: {
    main: {
      import: './index.js',
      filename: '[name].[fullhash].js',
    },
  },
  optimization: {
    chunkIds: 'named',
  },
  output: {
    filename: '[name].js',
    chunkFilename: '[name].js',
  },
};
