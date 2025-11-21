const path = require('path');

module.exports = {
  optimization: {
    chunkIds: "named",
    moduleIds: "named",
    splitChunks: false
  },
  output: {
    chunkFilename: "worker.js",
  },
  node: {
    __dirname: false,
    __filename: false,
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        type: 'javascript/auto',
        resolve: {
          alias: {
            'somefakemodule': path.resolve(__dirname, "./node_modules/corejs")
          }
        }
      },
    ]
  }
};