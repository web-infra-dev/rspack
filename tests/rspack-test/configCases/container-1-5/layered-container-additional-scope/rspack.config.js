const { container, DefinePlugin } = require('@rspack/core');

module.exports = [false, true].map((passScopeMap) => {
  const name = passScopeMap ? 'layered_with_map' : 'layered_without_map';
  const filename = `${name}.js`;
  return {
    target: 'async-node',
    entry: { [`${name}-host`]: './index.js' },

    resolve: {
      alias: {
        'paired-runtime':
          require.resolve('@module-federation/runtime-tools-layers'),
      },
    },
    experiments: { layers: true },
    module: { rules: [{ test: /provider\.js$/, layer: 'server' }] },
    output: {
      uniqueName: name,
      filename: '[name].js',
      chunkFilename: `${name}/[id].js`,
    },
    externals: { './remoteEntry.js': `commonjs ./${filename}` },
    plugins: [
      new DefinePlugin({ __PASS_SCOPE_MAP__: JSON.stringify(passScopeMap) }),
      new container.ModuleFederationPlugin({
        name,
        filename,
        library: { type: 'commonjs-module' },
        implementation:
          require.resolve('@module-federation/runtime-tools-layers'),
        exposes: {
          './provider': './provider.js',
          './plain': './plain.js',
          './unused': './unused.js',
        },
        remotes: {
          unused: {
            external:
              'promise Promise.resolve({ init() {}, get() { return () => null; } })',
            shareScope: ['ref-a', 'ref-b'],
          },
        },
        shared: {
          plain: {
            import: './plain.js',
            shareScope: 'scalar',
            version: '1.0.0',
          },
          lib: {
            import: './provider.js',
            shareKey: 'lib',
            shareScope: 'extra',
            layer: 'server',
            version: '1.0.0',
          },
        },
      }),
    ],
  };
});
