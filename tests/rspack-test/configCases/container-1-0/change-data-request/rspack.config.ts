import { defineConfig, definePlugin } from '@rspack/cli';
import { container } from '@rspack/core';

const { ModuleFederationPluginV1: ModuleFederationPlugin } = container;

export default defineConfig({
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
    definePlugin(function (compiler) {
      compiler.hooks.thisCompilation.tap(
        'ChangeDataRequest',
        (_compilation, { normalModuleFactory }) => {
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
    }),
  ],
});
