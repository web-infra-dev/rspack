const { experiments: { RslibPlugin } } = require('@rspack/core')

module.exports = {
  externals: {
    fs: 'module fs',
    path: 'module path',
  },
  plugins: [
    new RslibPlugin()
  ]
}