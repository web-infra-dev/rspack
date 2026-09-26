const path = require('node:path');

module.exports = {
  context: __dirname,
  resolve: {
    alias: {
      '@': path.join(__dirname, 'src'),
    },
  },
};
