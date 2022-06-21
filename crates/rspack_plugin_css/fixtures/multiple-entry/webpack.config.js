import Self from '../../../src'

module.exports = {
  entry: {
    'main-one': './index-one.js',
    'main-two': './index-two.js',
  },
  module: {
    rules: [
      {
        test: /\.css$/,
        use: [Self.loader, 'css-loader'],
      },
    ],
  },
  plugins: [
    new Self({
      filename: '[name].css',
    }),
  ],
}
