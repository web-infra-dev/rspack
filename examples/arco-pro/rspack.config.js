const lessLoader = require('@rspack/plugin-less').default;
const postcssLoader = require('@rspack/plugin-postcss');
const path = require('path');

/**
 * @type {import('webpack').Configuration}
 */
module.exports = {
  mode: 'production',
  context: __dirname,
  entry: { main: './src/index.tsx' },
  devServer: {
    port: 5555
  },
  devtool: false,
  builtins: {
    html: [{
      template: './index.html',
      publicPath: '/'
    }],
    define: { 'process.env.NODE_ENV': JSON.stringify('production') },
    react: {
      // development: true,
      // refresh: true,
    },
    progress: {},
    treeShaking: false,
  },
  module: {
    rules:
      [
        {
          test: /\.less$/,
          uses:
            [
              { loader: postcssLoader, options: { modules: true } },
              { loader: lessLoader },
            ],
          type: 'css'
        },
        { test: /\.svg$/, uses: [{ loader: './svg-loader.js' }], type: 'jsx' }
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
