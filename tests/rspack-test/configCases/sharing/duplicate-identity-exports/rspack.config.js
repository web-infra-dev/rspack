const { ModuleFederationPlugin } = require('@rspack/core').container;
module.exports = {
  target: 'async-node',
  module: { rules: [{ test: /(?:first|second)\.js$/, sideEffects: false }] },
  plugins: [
    new ModuleFederationPlugin({
      name: 'duplicate_identity_exports',
      manifest: true,
      shared: {
        './first.js': {
          shareKey: 'duplicate',
          version: '1.0.0',
          treeShaking: { mode: 'runtime-infer', usedExports: ['a'] },
        },
        './second.js': {
          shareKey: 'duplicate',
          version: '2.0.0',
          treeShaking: { mode: 'runtime-infer', usedExports: ['b'] },
        },
      },
    }),
  ],
};
