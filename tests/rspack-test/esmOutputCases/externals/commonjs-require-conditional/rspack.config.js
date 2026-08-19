module.exports = {
  entry: {
    safe: './entry-safe.js',
    unsafe: './entry-unsafe.js',
  },
  externals: {
    external: 'commonjs external',
  },
};
