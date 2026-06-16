const { ModuleFederationPlugin } = require('@rspack/core').container;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
  experiments: {
    runtimeMode: 'rspack',
  },
  target: 'async-node',
  output: {
    publicPath: '/assets/',
  },
  plugins: [
    new ModuleFederationPlugin({
      name: 'runtime_mode_public_path',
      shared: {
        'shared-lib': {
          requiredVersion: '*',
          treeShaking: {
            mode: 'runtime-infer',
          },
        },
      },
    }),
  ],
};
