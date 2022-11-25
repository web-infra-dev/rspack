const lessLoader = require('@rspack/less-loader');
const postcssLoader = require('@rspack/postcss-loader');
const path = require('path');

/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
  mode: 'production',
  context: __dirname,
  entry: { main: './src/index.tsx' },
  devServer: {
    port: 5555,
    hot: false
  },
  devtool: false,
  builtins: {
    html: [{
      template: './index.html',
      publicPath: '/'
    }],
    define: { 'process.env.NODE_ENV': JSON.stringify('production') },
    react: {
      development: false,
    },
    progress: {},
    treeShaking: false,
  },
  module: {
    rules:
      [
        {
          test: /\.less$/,
          use:
            [
              { loader: postcssLoader, options: { modules: true } },
              { loader: lessLoader },
            ],
          type: 'css'
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
