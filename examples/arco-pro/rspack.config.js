const path = require('path');

/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
  stats: false,
  context: __dirname,
  entry: { main: './src/index.tsx' },
  devServer: {
    port: 5555,
    webSocketServer: 'sockjs'
  },
  devtool: false,
  builtins: {
    html: [{
      template: './index.html',
      publicPath: '/'
    }],
    react: {
      development: true,
      refresh: true,
    },
    progress: {},
    treeShaking: true,
    sideEffects: true,
    noEmitAssets: false
  },
  cache: false,
  module: {
    rules:
      [
        {
          test: /\.less$/,
          use:
            [
              { loader: 'less-loader' },
            ],
          type: 'css'
        },
        {
          test: /\.module\.less$/,
          use:
            [
              { loader: 'less-loader' },
            ],
          type: 'css/module'
        },
        { test: /\.svg$/, use: [{ loader: './svg-loader.js' }], type: 'jsx' }
      ]
  },
  resolve: { alias: { '@': path.resolve(__dirname, 'src') } },
  output: {
    publicPath: '/'
  },
  infrastructureLogging: {
    debug: false
  },
}
