module.exports = {
  output: {
    chunkFilename: 'chunks/[name].mjs',
  },
  externals: {
    'relative-external': 'node-commonjs ./external.cjs',
    'package-external': 'node-commonjs shadow-pkg',
  },
};
