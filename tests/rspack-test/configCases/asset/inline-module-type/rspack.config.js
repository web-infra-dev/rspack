module.exports = {
  module: {
    rules: [
      {
        test: /\.txt$/,
        type: 'asset/source',
        resolve: { alias: { value: require.resolve('./value.js') } },
      },
    ],
  },
};
