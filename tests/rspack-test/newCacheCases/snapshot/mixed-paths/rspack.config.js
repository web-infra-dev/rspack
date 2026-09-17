const path = require('node:path');
module.exports = {
  context: __dirname,
  experiments: { newCache: { module: true, loader: false } },
  cache: {
    type: 'persistent',
    snapshot: {
      immutablePaths: [path.join(__dirname, 'immutable')],
      unmanagedPaths: [path.join(__dirname, 'immutable/mutable')],
      managedPaths: [/^(.+?[\\/]packages[\\/])/],
    },
  },
  module: {
    rules: [{ test: /consumer-.*\.js$/, loader: './dependency-loader.js' }],
  },
};
