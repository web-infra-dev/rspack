const path = require('node:path');

module.exports = {
  context: __dirname,
  resolve: {
    modules: [__dirname, 'node_modules'],
    alias: {
      'dir/$': path.join(__dirname, 'missing-target'),
      dir: path.join(__dirname, 'redirected'),
    },
  },
};
