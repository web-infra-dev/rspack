const path = require('path');
const { default: HtmlPlugin } = require('@rspack/plugin-html');

const prod = process.env.NODE_ENV === 'production';

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
  mode: prod ? 'production' : 'development',
  devtool: prod ? false : 'source-map',
  builtins: {
    progress: {},
    treeShaking: true,
    sideEffects: true,
    noEmitAssets: false,
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
  plugins: [
    new HtmlPlugin({
      title: 'Arco Pro App',
      template: path.join(__dirname, 'index.html'),
      favicon: path.join(__dirname, 'public', 'favicon.ico'),
    }),
  ],
  infrastructureLogging: {
    debug: false
  }
}
