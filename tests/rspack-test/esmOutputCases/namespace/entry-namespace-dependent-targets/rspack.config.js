module.exports = ['dependent', 'before'].map(name => ({
  name,
  entry: name === 'dependent' ? './index.js' : './before.js',
  output: {
    filename: `[name].${name}.mjs`,
    chunkFilename: `[name].${name}.mjs`,
  },
}));
