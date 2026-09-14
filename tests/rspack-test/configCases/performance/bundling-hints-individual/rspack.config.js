const path = require('node:path');
const base = require('../bundling-hints/rspack.config');

module.exports = [
  'asyncChunkWaterfalls',
  'embeddedSourceMaps',
  'inlinedAssets',
  'topLevelThis',
].map((option) => ({
  ...base,
  context: path.resolve(__dirname, '../bundling-hints'),
  performance: { all: false, hints: 'warning', [option]: true },
}));
