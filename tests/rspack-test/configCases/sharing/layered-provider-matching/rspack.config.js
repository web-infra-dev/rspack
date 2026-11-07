const { ProvideSharedPlugin } = require('@rspack/core').sharing;

module.exports = ['server', 'client'].map((layer) => ({
  experiments: { layers: true },
  module: {
    rules: [
      { test: /index\.js$/, exclude: /node_modules/, layer },
      { test: /first\.js$/, layer: 'a' },
      { test: /second\.js$/, layer: 'a)b' },
    ],
  },
  plugins: [
    new ProvideSharedPlugin({
      enhanced: true,
      provides: [
        { 'b)c': { shareKey: 'collision-a', layer: 'a', version: '1.0.0' } },
        { c: { shareKey: 'collision-b', layer: 'a)b', version: '1.0.0' } },
        { package: { shareKey: 'fallback', version: '1.0.0' } },
        {
          package: {
            shareKey: 'exact',
            shareScope: 'first',
            layer: 'server',
            version: '1.0.0',
          },
        },
        {
          package: {
            shareKey: 'exact',
            shareScope: 'first',
            layer: 'server',
            version: '2.0.0',
          },
        },
        {
          package: { shareKey: 'other', shareScope: 'second', layer: 'server' },
        },
        {
          'package/': { shareKey: 'short/', layer: 'server', version: '1.0.0' },
        },
        { 'package/': { shareKey: 'fallback-short/', version: '1.0.0' } },
        { 'package/feature/': { shareKey: 'long/', version: '1.0.0' } },
      ],
    }),
  ],
}));
