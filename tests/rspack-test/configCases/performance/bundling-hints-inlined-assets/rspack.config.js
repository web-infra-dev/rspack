module.exports = {
  performance: { inlinedAssets: true, hints: 'warning' },
  module: {
    rules: [
      {
        test: /asset\.txt$/,
        type: 'asset/inline',
        use: ['./asset-loader.js'],
        oneOf: [
          { resourceQuery: /resource/, type: 'asset/resource' },
          { resourceQuery: /auto/, type: 'asset' },
        ],
      },
    ],
  },
};
