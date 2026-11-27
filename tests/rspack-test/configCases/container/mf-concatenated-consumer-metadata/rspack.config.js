const path = require('path');
const { ModuleFederationPlugin } = require('@rspack/core').container;

module.exports = {
  mode: 'production',
  target: 'node',
  entry: { a: './a.js', c: './c.js', loader: './loader.js' },
  output: { filename: '[name].js', uniqueName: 'mf-concatenated-consumers' },
  resolve: {
    alias: {
      '@module-federation/node/runtimePlugin': path.resolve(
        __dirname,
        'node-runtime-plugin.js',
      ),
    },
  },
  optimization: {
    concatenateModules: true,
    minimize: true,
    moduleIds: 'deterministic',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'host',
      remotes: {
        remote: 'promise Promise.resolve({ init() {}, get() {} })',
      },
      runtimePlugins: ['@module-federation/node/runtimePlugin'],
    }),
  ],
};
