const { container, DefinePlugin } = require('@rspack/core');

const pairs = [
  ['new-host', 'layered', 'old'],
  ['new-remote', 'old', 'layered'],
  ['v1-remote', 'layered', 'v1'],
  ['v1-host', 'v1', 'layered'],
  ['transitive', 'old', 'layered', true],
];

const app = (id, kind, remotes, host, version = '1.0.0') => {
  const layered = kind === 'layered';
  const Plugin =
    kind === 'v1'
      ? container.ModuleFederationPluginV1
      : container.ModuleFederationPlugin;
  return {
    target: 'async-node',
    entry: { [`${id}/main`]: host ? './index.js' : './empty.js' },
    output: {
      ...(host ? { library: { type: 'commonjs2' } } : {}),
      uniqueName: id,
      filename: '[name].js',
      chunkFilename: `${id}/[id].js`,
    },
    ...(layered
      ? {
          experiments: { layers: true },
          module: { rules: [{ test: /layered\.js$/, layer: 'server' }] },
        }
      : {}),
    resolve: { alias: { 'singleton-lib': require.resolve('./singleton.js') } },
    plugins: [
      new DefinePlugin({
        __PROVIDER__: JSON.stringify(
          version === '2.0.0' ? 'last' : host ? 'host' : 'middle',
        ),
      }),
      new Plugin({
        name: id.replace(/-/g, '_'),
        ...(layered
          ? {
              implementation:
                require.resolve('@module-federation/runtime-tools-layers'),
            }
          : {}),
        library: { type: 'commonjs-module' },
        filename: `${id}/remoteEntry.js`,
        ...(host
          ? {}
          : {
              exposes: {
                './value': './value.js',
                ...(remotes ? { './unused': './unused.js' } : {}),
                ...(layered ? { './layered': './layered.js' } : {}),
              },
            }),
        remotes,
        shared: {
          'singleton-lib': { singleton: true, requiredVersion: false, version },
          ...(layered
            ? {
                layered: {
                  import: './layered.js',
                  layer: 'server',
                  version: '1.0.0',
                },
              }
            : {}),
        },
      }),
    ],
  };
};

module.exports = pairs.flatMap(([name, host, remote, transitive]) =>
  [false, true].flatMap((reference) => {
    const id = `${name}-${reference ? 'reference' : 'mixed'}`;
    const remoteRef = (target, shareScope) => ({
      external: `promise globalThis.__mfPeerLoad(${JSON.stringify(target)})`,
      shareScope,
    });
    return [
      ...(transitive
        ? [app(`${id}-last`, 'old', undefined, false, '2.0.0')]
        : []),
      app(
        `${id}-middle`,
        reference && remote === 'layered' ? 'old' : remote,
        transitive ? { last: remoteRef(`${id}-last`, 'other') } : undefined,
        false,
      ),
      app(
        id,
        reference && host === 'layered' ? 'old' : host,
        { peer: remoteRef(`${id}-middle`, 'default') },
        true,
        transitive ? '1.0.0' : '1.1.0',
      ),
    ];
  }),
);
