const path = require('node:path');
const { container, CopyRspackPlugin, DefinePlugin } = require('@rspack/core');

module.exports = (_, { testPath }) =>
  [
    ['scalar-version-first', 'version-first', 'default', false],
    ['scalar-remote-only', 'version-first', 'default', false, true],
    ['ordered-loaded-first', 'loaded-first', ['primary', 'default'], false],
    [
      'ordered-loaded-first-initial',
      'loaded-first',
      ['primary', 'default'],
      true,
    ],
    ['ordered-version-first', 'version-first', ['primary', 'default'], false],
    [
      'ordered-version-first-initial',
      'version-first',
      ['primary', 'default'],
      true,
    ],
  ].map(([name, shareStrategy, shareScope, eager, remoteOnly]) => ({
    target: 'async-node',
    entry: eager ? './initial.js' : './index.js',
    output: {
      path: path.join(testPath, name),
      filename: 'main.js',
      chunkFilename: '[id].js',
      uniqueName: name,
    },
    plugins: [
      new container.ModuleFederationPlugin({
        name,
        shareStrategy,
        remoteType: 'commonjs-module',
        remotes: {
          remote: { external: './remote.cjs', shareScope },
          // Scalar version-first must not wait on an unused container promise.
          ...(typeof shareScope === 'string'
            ? {
                pending: {
                  external: 'promise new Promise(() => {})',
                  shareScope: 'pending',
                },
              }
            : {}),
        },
        shared: {
          lib: {
            import: remoteOnly ? false : './local.js',
            version: '1.0.0',
            requiredVersion: false,
            shareScope,
            eager,
          },
        },
      }),
      new CopyRspackPlugin({ patterns: [{ from: './remote.cjs' }] }),
      new DefinePlugin({
        __HAS_PENDING_REMOTE__: JSON.stringify(typeof shareScope === 'string'),
        __EXPECT_REMOTE_PROVIDER__: JSON.stringify(
          shareStrategy === 'version-first' &&
            (Array.isArray(shareScope) || remoteOnly === true),
        ),
      }),
    ],
  }));
