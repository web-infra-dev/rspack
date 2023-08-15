const { WebpackManifestPlugin } = require('rspack-manifest-plugin');
module.exports = {
  mode: 'production',
  devtool:false,
  target: ['web','es5'],
  optimization: {
    minimize:false,
    concatenateModules:false,
    mangleExports:false,
    moduleIds: 'named'
  },
  stats: 'verbose',
  entry: {
    main: './index.js'
  },
  output: {
    path: 'rspack-dist',
    publicPath: ''
  },
  plugins: [ new WebpackManifestPlugin()]
}