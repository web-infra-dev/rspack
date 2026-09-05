module.exports = {
  entry: {
    entryA: './entries/entryA.js',
    entryB: './entries/entryB.js',
  },
  output: {
    filename: '[name].js',
  },
  // Keep otherwise-unused entry exports in the chunk graph for this topology assertion.
  optimization: {
    innerGraph: false,
  },
};
