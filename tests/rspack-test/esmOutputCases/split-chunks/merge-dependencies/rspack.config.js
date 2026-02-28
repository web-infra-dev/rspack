module.exports = {
  optimization: {
    splitChunks: {
      cacheGroups: {
        a: {
          test: /a\.js/,
        },
        b: {
          test: /b\.js/,
        },
      }
    }
  }
}