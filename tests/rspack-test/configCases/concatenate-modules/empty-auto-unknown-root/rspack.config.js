module.exports = {
  mode: 'production',
  entry: ['./index.js', './compiler.js'],
  output: { library: { type: 'commonjs2' } },
  optimization: {
    minimize: false,
    concatenateModules: true,
    inlineExports: false,
    // Keep the named observation on the barrel instead of redirecting it to the empty leaf.
    sideEffects: false,
  },
  stats: {
    modules: true,
    nestedModules: true,
    optimizationBailout: true,
    providedExports: true,
  },
};
