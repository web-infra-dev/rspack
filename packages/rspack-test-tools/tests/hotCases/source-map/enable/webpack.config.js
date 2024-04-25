module.exports = {
  entry: { 
    main: './index.js',
  },
  devtool: 'cheap-source-map',
  externalsPresets: {
    node: true,
  },
  node: {
    __dirname: false,
  }
}