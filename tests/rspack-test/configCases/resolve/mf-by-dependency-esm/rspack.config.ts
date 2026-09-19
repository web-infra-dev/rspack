import { defineConfig } from '@rspack/cli';

import { rspack } from '@rspack/core';

export default defineConfig({
  module: {
    rules: [
      {
        type: 'javascript/auto',
        test: /\.js$/,
        use: {
          loader: 'builtin:swc-loader',
          options: {
            detectSyntax: 'auto',
            jsc: {
              parser: {
                exportDefaultFrom: true,
              },
            },
            module: {
              type: 'commonjs',
              strict: false,
              strictMode: false,
              noInterop: false,
              lazy: false,
              allowTopLevelThis: true,
              ignoreDynamic: true,
            },
          },
        },
      },
    ],
  },
  resolve: {
    extensions: ['.native.js', '.js'],
  },
  plugins: [
    new rspack.container.ModuleFederationPluginV1({
      name: 'test',
      shared: {
        pkg: {
          singleton: true,
          eager: true,
        },
      },
    }),
  ],
});
