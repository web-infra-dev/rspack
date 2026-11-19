const { ModuleFederationPlugin } = require('@rspack/core').container;

module.exports = {
  mode: 'development',
  target: 'async-node',
  optimization: { minimize: false },
  plugins: [
    new ModuleFederationPlugin({
      implementation:
        require.resolve('@module-federation/runtime-tools-layers'),
      name: 'fallback_scope_shapes',
      manifest: true,
      library: { type: 'commonjs-module' },
      shared: {
        one: {
          shareKey: 'shape',
          shareScope: 'default',
          requiredVersion: false,
          treeShaking: { mode: 'runtime-infer' },
        },
        two: {
          shareKey: 'shape',
          shareScope: ['default'],
          requiredVersion: false,
          treeShaking: { mode: 'runtime-infer' },
        },
        'ambiguous-a': {
          shareKey: 'ambiguous',
          requiredVersion: false,
          treeShaking: { mode: 'runtime-infer' },
        },
        'ambiguous-b': {
          shareKey: 'ambiguous',
          requiredVersion: false,
          treeShaking: { mode: 'runtime-infer' },
        },
      },
    }),
  ],
};
