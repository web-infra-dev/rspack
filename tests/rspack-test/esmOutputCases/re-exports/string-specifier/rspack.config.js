module.exports = {
  module: {
    rules: [
      {
        test: /\.json$/,
        type: 'json'
      }
    ]
  },
  optimization: {
    splitChunks: { 
      cacheGroups: {
        lib: {
          test: /lib\.js/
        }
      }
    }
  }
};
