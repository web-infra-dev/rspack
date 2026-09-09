module.exports = {
  mode: 'development',
  cache: false,
  incremental: true,
  entry: {
    a: ['./a.js', './bar.js'],
    b: ['./b.js', './bar.js'],
  },
  output: { filename: '[name].js' },
  optimization: { concatenateModules: true, usedExports: true },
};
