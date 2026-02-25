module.exports = {
  optimization: {
    runtimeChunk: false,
    splitChunks: {
      cacheGroups: {
        other: {
          test: /other\.js/,
          name: 'other'
        }
      }
    }
  }
}