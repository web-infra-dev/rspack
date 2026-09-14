const path = require('node:path');
const base = require('../bundling-hints/rspack.config');

module.exports = {
  ...base,
  context: path.resolve(__dirname, '../bundling-hints'),
  performance: { all: true, hints: 'error' },
};
