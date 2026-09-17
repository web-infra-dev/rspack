import { container } from '@rspack/core';

const { ModuleFederationPluginV1: ModuleFederationPlugin } = container;

/** @type {import("@rspack/core").Configuration} */
export default {
  mode: 'development',
  plugins: [
    new ModuleFederationPlugin({
      name: 'A',
      filename: 'container-a.js',
      library: {
        type: 'commonjs-module',
      },
      exposes: {
        '.': './a',
      },
      remoteType: 'commonjs-module',
      remotes: {
        A: './container-a.js',
      },
      shared: ['myX'],
    }),
    function (compiler) {
      compiler.hooks.thisCompilation.tap(
        'ChangeDataRequest',
        (compilation, { normalModuleFactory }) => {
          normalModuleFactory.hooks.beforeResolve.tap(
            'ChangeDataRequest',
            (data) => {
              if (data.request === 'myA') {
                data.request = 'A';
              }
              if (data.request === 'myX') {
                data.request = './x';
              }
            },
          );
        },
      );
    },
  ],
};
