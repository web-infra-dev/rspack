const path = require('node:path');
const { container, CopyRspackPlugin, sharing } = require('@rspack/core');

module.exports = (_, { testPath }) =>
  [
    { name: 'scalar-version-first', shareScope: 'default' },
    { name: 'unlayered-array-catalog' },
    { name: 'unlayered-array-paired', paired: true },
    { name: 'layered-array-sync', paired: true, layered: true },
  ].map(({ name, shareScope = ['primary', 'default'], paired, layered }) => ({
    ...(layered
      ? {
          experiments: { layers: true },
          module: { rules: [{ test: /layer-marker\.js$/, layer: 'test' }] },
        }
      : {}),
    target: 'async-node',
    entry: './index.js',
    output: {
      path: path.join(testPath, name),
      filename: 'main.js',
      chunkFilename: '[id].js',
      uniqueName: name,
      library: { type: 'commonjs2' },
    },
    plugins: [
      // This provider opts only the layered control into declared scope dispatch.
      ...(layered
        ? [
            new sharing.ProvideSharedPlugin({
              enhanced: true,
              provides: {
                './layer-marker.js': {
                  shareKey: 'layer-marker',
                  layer: 'test',
                  version: '1.0.0',
                },
              },
            }),
          ]
        : []),
      new container.ModuleFederationPlugin({
        name,
        shareStrategy: 'version-first',
        implementation: paired
          ? require.resolve('@module-federation/runtime-tools-layers')
          : undefined,
        remoteType: 'commonjs-module',
        remotes: { remote: { external: './remote.cjs', shareScope } },
        shared: {
          lib: {
            import: './local.js',
            version: '1.0.0',
            requiredVersion: false,
            shareScope,
            eager: true,
          },
        },
      }),
      new CopyRspackPlugin({ patterns: [{ from: './remote.cjs' }] }),
    ],
  }));
