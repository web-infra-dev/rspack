const path = require('path');

/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
  context: __dirname,
  mode: 'development',
  entry: {
    main: {
      import: ["./src/index.js"],
    }
  },
  output: {
    publicPath: '/',
    // filename: '[name].[contenthash:8][ext]',
  },
  devServer: {
    webSocketServer: 'ws',
    hot: true,
  },
  module: {
    rules: [
      {
        test: /\.less$/,
        type: 'css'
      },
    ],
    parser: {
      asset: {
        dataUrlCondition: {
          maxSize: 1,
        },
      },
    },
  },
  infrastructureLogging: {
    debug: true,
  },
  builtins: {
    html: [{
      template: './index.html'
    }],
    define: {
      'process.env.NODE_ENV': "'development'"
    },
    progress: {},
    react: {
      development: true,
      refresh: true,
    },
    styledJsx: true
  },
};
