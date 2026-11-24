const { container, sharing } = require('@rspack/core');

const external = `Promise.resolve({
  init(scope, initScope, options) {
    globalThis.__mfScopeInits ||= [];
    globalThis.__mfScopeInits.push({ scope, keys: options.shareScopeKeys });
  },
  get() { return Promise.resolve(() => 'remote'); }
})`;

module.exports = {
  experiments: { layers: true },
  module: { rules: [{ test: /layer-marker\.js$/, layer: 'test' }] },
  target: 'async-node',
  plugins: [
    new container.ModuleFederationPlugin({
      name: 'remote-scope-isolation',
      implementation:
        require.resolve('@module-federation/runtime-tools-layers'),
      remoteType: 'promise',
      remotes: {
        ordered: { external, shareScope: ['secondary', 'primary'] },
        isolated: { external, shareScope: 'isolated' },
      },
    }),
    new sharing.ProvideSharedPlugin({
      enhanced: true,
      provides: {
        // This provider opts the build into declared scope dispatch.
        './layer-marker.js': {
          shareKey: 'layer-marker',
          layer: 'test',
          version: '1.0.0',
        },
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
