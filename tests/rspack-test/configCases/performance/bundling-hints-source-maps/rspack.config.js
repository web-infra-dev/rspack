module.exports = [
  ['production', 'inline-source-map'],
  ['production', 'eval-source-map'],
  ['production', 'source-map'],
  ['development', 'inline-source-map'],
  ['production', false],
  [undefined, 'inline-source-map'],
  [undefined, 'eval-source-map'],
  ['none', 'inline-source-map'],
].map(([mode, devtool]) => ({
  mode,
  devtool,
  performance: { embeddedSourceMaps: true, hints: 'warning' },
  plugins: [
    (compiler) => {
      // Remove the config-case harness's production mode before defaults run.
      if (mode === undefined) delete compiler.options.mode;
    },
  ],
}));
