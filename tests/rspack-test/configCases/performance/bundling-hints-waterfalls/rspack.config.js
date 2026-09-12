const path = require('node:path');

module.exports = [
  ['./index.js', './leaf.js'],
  ['./index.js', './shared.js'],
  ['./shallow.js', './leaf.js'],
].map(([entry, shared]) => ({
  entry,
  mode: 'production',
  performance: { asyncChunkWaterfalls: true, hints: 'warning' },
  resolve: { alias: { shared: path.resolve(__dirname, shared) } },
}));
