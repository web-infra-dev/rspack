const rspack = require('@rspack/core')

module.exports = {
  optimization: {
    splitChunks: {
      cacheGroups: {
        splitMain: {
          test: /index\.js/
        }
      }
    }
  },
  plugins: [
    new rspack.experiments.RslibPlugin()
  ]
}