const path = require('path');

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  target: 'node',
  resolve: {
    alias: {
      // The function ModuleFederationPlugin injects as the default runtime.
      'default-runtime': path.resolve(
        __dirname,
        '../../../../../packages/rspack/dist/moduleFederationDefaultRuntime.js',
      ),
      'bundler-runtime':
        require.resolve('@module-federation/runtime-tools/webpack-bundler-runtime'),
    },
  },
};
