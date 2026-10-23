const path = require('node:path');

module.exports = {
  context: __dirname,
  resolve: {
    alias: {
      dir: [],
      ignored: false,
      nested$: path.join(__dirname, 'src', 'does-not-exist'),
      '@': path.join(__dirname, 'src'),
      '~': __dirname,
      'special-dir': path.join(__dirname, 'src', '[dir]'),
    },
  },
};
