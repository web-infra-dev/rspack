const { ContainerReferencePlugin, ModuleFederationPlugin } =
  require('@rspack/core').container;

module.exports = {
  target: 'async-node',
  plugins: [
    new ModuleFederationPlugin({ name: 'host' }),
    new ContainerReferencePlugin({
      remoteType: 'promise',
      enhanced: true,
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
