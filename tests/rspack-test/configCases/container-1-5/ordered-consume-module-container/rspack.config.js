const path = require('path');
const { ModuleFederationPlugin } = require('@rspack/core').container;
const { ConsumeSharedPlugin } = require('@rspack/core').sharing;

module.exports = (_, { testPath }) =>
  ['async-only', 'initial'].map((name) => ({
    name,
    target: 'node',
    entry:
      name === 'initial'
        ? { index: './index.js', ordered: './ordered.js' }
        : { index: './index.js' },
    experiments: { outputModule: true },
    output: {
      path: path.join(testPath, name),
      filename: '[name].mjs',
      chunkFilename: '[id].mjs',
      module: true,
      uniqueName: `ordered-consume-module-container-${name}`,
    },
    resolve: {
      alias: {
        lib: require.resolve('../ordered-consume-startup/node_modules/lib'),
      },
    },
    optimization: { runtimeChunk: 'single' },
    plugins: [
      new ModuleFederationPlugin({
        name: `ordered_container_${name.replace('-', '_')}`,
        filename: 'remoteEntry.mjs',
        library: { type: 'module' },
        exposes: { './exposed': './exposed.js' },
      }),
      new ConsumeSharedPlugin({
        enhanced: true,
        consumes: {
          lib: {
            shareScope: ['primary', 'default'],
            eager: true,
            requiredVersion: false,
          },
        },
      }),
    ],
  }));
