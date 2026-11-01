module.exports = {
  output: { chunkFilename: '[name].[contenthash].js' },
  optimization: {
    moduleIds: 'named',
    chunkIds: 'named',
    concatenateModules: false,
    splitChunks: {
      cacheGroups: {
        default: false,
        defaultVendors: false,
        shared: {
          test: /[\\/]dep\.js$/,
          name: 'shared',
          chunks: 'async',
          enforce: true,
        },
      },
    },
  },
};
