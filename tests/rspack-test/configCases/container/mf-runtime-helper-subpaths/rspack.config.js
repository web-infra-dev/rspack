const path = require('node:path');
const { container } = require('@rspack/core');

module.exports = {
  target: 'node',
  externals: { './container.js': 'commonjs ./container.js' },
  plugins: [
    new container.ModuleFederationPlugin({
      name: 'runtime_helper_subpaths',
      implementation: require.resolve('@module-federation/runtime-tools'),
      filename: 'container.js',
      library: { type: 'commonjs-module' },
      exposes: { './module': './module.js' },
      runtimePlugins: [path.join(__dirname, 'plugin.js')],
    }),
  ],
};
