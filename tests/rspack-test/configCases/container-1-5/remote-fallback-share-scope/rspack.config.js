const {
  container: { ModuleFederationPlugin },
  sharing: { ProvideSharedPlugin },
} = require('@rspack/core');

module.exports = {
  experiments: { layers: true },
  module: { rules: [{ test: /layer-marker\.js$/, layer: 'test' }] },
  target: 'async-node',
  plugins: [
    new ProvideSharedPlugin({
      enhanced: true,
      provides: {
        // This provider opts the build into declared scope dispatch.
        './layer-marker.js': {
          shareKey: 'layer-marker',
          layer: 'test',
          version: '1.0.0',
        },
      },
    }),
    new ModuleFederationPlugin({
      name: 'remote-fallback-share-scope',
      implementation:
        require.resolve('@module-federation/runtime-tools-layers'),
      remoteType: 'promise',
      remotes: {
        remote: {
          external: [
            'Promise.reject(new Error("unavailable"))',
            `Promise.resolve({
              init: (_shareScope, _initScope, options) => {
                globalThis.__shareScopeKeys = options.shareScopeKeys;
              },
              get: () => Promise.resolve(() => globalThis.__shareScopeKeys)
            })`,
          ],
          shareScope: ['scope1', 'scope2'],
        },
      },
    }),
  ],
};
