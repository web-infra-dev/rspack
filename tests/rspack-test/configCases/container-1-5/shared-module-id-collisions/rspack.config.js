const {
  container: { ModuleFederationPlugin },
  sources: { RawSource },
} = require('@rspack/core');
module.exports = {
  experiments: { layers: true },
  optimization: { concatenateModules: false },
  plugins: [
    new ModuleFederationPlugin({
      name: 'shared_module_id_collisions',
      shared: {
        first: {
          import: './shared.js',
          shareKey: 'c',
          layer: 'a) b',
          version: '1.0.0',
          requiredVersion: false,
          eager: true,
        },
        second: {
          import: './shared.js',
          shareKey: 'b) c',
          layer: 'a',
          version: '1.0.0',
          requiredVersion: false,
          eager: true,
        },
      },
    }),
    {
      apply(compiler) {
        compiler.hooks.thisCompilation.tap('identities', (compilation) => {
          compilation.hooks.processAssets.tap('identities', () => {
            const modules = [...compilation.modules].map((module) => ({
              id: module.identifier(),
              name: module.readableIdentifier(compilation.requestShortener),
            }));
            compilation.emitAsset(
              'identities.json',
              new RawSource(JSON.stringify(modules)),
            );
          });
        });
      },
    },
  ],
};
