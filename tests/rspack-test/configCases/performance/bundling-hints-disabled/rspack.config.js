/** @type {import('@rspack/core').Configuration[]} */
module.exports = [
  false,
  { hints: 'warning' },
  { all: false, hints: 'warning' },
  { all: true, hints: false },
  {
    all: true,
    hints: 'warning',
    asyncChunkWaterfalls: false,
    embeddedSourceMaps: false,
    inlinedAssets: false,
    topLevelThis: false,
  },
].map((performance) => ({
  mode: 'production',
  devtool: 'inline-source-map',
  performance,
  module: {
    rules: [{ test: /big\.svg$/, type: 'asset/inline' }],
  },
  optimization: { minimize: false },
}));
