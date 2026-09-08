module.exports = [
  ['production', 'inline-source-map'],
  ['production', 'eval-source-map'],
  ['production', 'source-map'],
  ['development', 'inline-source-map'],
  ['production', false],
].map(([mode, devtool]) => ({
  mode,
  devtool,
  performance: { embeddedSourceMaps: true, hints: 'warning' },
}));
