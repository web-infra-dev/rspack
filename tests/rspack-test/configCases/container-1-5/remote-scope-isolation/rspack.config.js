const { container, sharing } = require('@rspack/core');

const external = `Promise.resolve({
  init(scope, initScope, options) {
    globalThis.__mfScopeInits ||= [];
    globalThis.__mfScopeInits.push({ scope, keys: options.shareScopeKeys });
  },
  get() { return Promise.resolve(() => 'remote'); }
})`;

module.exports = {
  target: 'async-node',
  plugins: [
    new container.ModuleFederationPlugin({ name: 'remote-scope-isolation' }),
    new container.ContainerReferencePlugin({
      enhanced: true,
      remoteType: 'promise',
      remotes: {
        ordered: { external, shareScope: ['secondary', 'primary'] },
        isolated: { external, shareScope: 'isolated' },
      },
    }),
    new sharing.ProvideSharedPlugin({
      enhanced: true,
      provides: {
        './primary.js': {
          shareScope: 'primary',
          shareKey: 'value',
          version: '1.0.0',
          eager: true,
        },
        './secondary.js': {
          shareScope: 'secondary',
          shareKey: 'value',
          version: '1.0.0',
          eager: true,
        },
        './isolated.js': {
          shareScope: 'isolated',
          shareKey: 'value',
          version: '1.0.0',
          eager: true,
        },
      },
    }),
  ],
};
